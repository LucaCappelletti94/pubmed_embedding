from typing import List, Dict, Set, Tuple
import os
import numpy as np
import pandas as pd
from downloaders import BaseDownloader
from userinput.utils import must_be_in_set
import compress_json


def get_versions() -> List[str]:
    """Returns available versions."""
    return [
        file_name.split(".")[0]
        for file_name in os.listdir("{}/versions".format(os.path.dirname(os.path.abspath(__file__))))
        if file_name.endswith(".json")
    ]


def get_metadata(
    version: str,
) -> Dict:
    """Returns pandas DataFrame with index.

    Parameters
    -------------------
    version: str
        The version of the index to retrieve.
    """
    version = must_be_in_set(version, get_versions(), "dataset version")
    return compress_json.local_load(f"versions/{version}.json", use_cache=True)


def get_index(
    version: str,
    downloads_directory: str
) -> pd.DataFrame:
    """Returns pandas DataFrame with index.

    Parameters
    -------------------
    version: str
        The version of the index to retrieve.
    downloads_directory: str
        The directory where to store the downloads
    """
    url = get_metadata(version)["index"]
    index_path = f"{version}_index.csv"
    BaseDownloader(
        downloads_directory=f"{downloads_directory}/{version}",
    ).download(urls=url, paths=index_path)
    df = pd.DataFrame(index_path, header=None)
    column = df.columns[0]
    df.reset_index(inplace=True)
    df.set_index(column, inplace=True)
    return df


def get_chunk_id_from_curie_id(
    curie_id: int,
    version: str,
) -> int:
    """Returns chunk ID containing embedding for provided curie ID.

    Parameters
    --------------------
    curie_id: int
        The curie ID to map to a chunk.
    version: str
        The version of the embedding to retrieve.
    """
    for chunk_id, chunk in enumerate(get_metadata(version)["chunk"]):
        if curie_id >= chunk["start"] and curie_id < chunk["end"]:
            return chunk_id
    raise ValueError(
        f"The provided curie ID {curie_id} for the dataset version {version} "
        "does not map to any known embedding chunk."
    )


def restrict_curie_id_to_chunk(
    curie_id: int,
    version: str,
) -> int:
    """Returns chunk ID containing embedding for provided curie ID.

    Parameters
    --------------------
    curie_id: int
        The curie ID to map to a chunk.
    version: str
        The version of the embedding to retrieve.
    """
    chunk_id = get_chunk_id_from_curie_id(curie_id, version)
    chunk = get_metadata(version)["chunk"][chunk_id]
    return chunk_id - chunk["start"]


def get_unique_chunk_ids_from_curie_ids(
    curie_ids: np.ndarray,
    version: str
) -> Set[int]:
    """Returns chunk IDs containing embedding for provided curie IDs.

    Parameters
    --------------------
    curie_ids: int
        The curie IDs to map to a chunks.
    version: str
        The version of the embedding to retrieve.
    """
    return {
        get_chunk_id_from_curie_id(curie_id, version)
        for curie_id in curie_ids
    }


def get_unique_urls_from_curie_ids(
    curie_ids: np.ndarray,
    version: str
) -> Tuple[List[str], List[int]]:
    """Returns unique chunk URLs and chunk IDs to embedding for provided curie IDs.

    Parameters
    --------------------
    curie_ids: int
        The curie IDs to map to a chunks.
    version: str
        The version of the embedding to retrieve.
    """
    chunks = get_metadata(version)["chunk"]
    chunk_ids = []
    urls = []
    for chunk_id in get_unique_chunk_ids_from_curie_ids(curie_ids, version):
        chunk_ids.append(chunk_id)
        urls.append(chunks[chunk_id]["url"])

    return urls, chunk_ids


def get_embedding_chunk_path_from_curie_id(
    curie_id: int,
    version: str,
    downloads_directory: str
) -> str:
    """Return path to embedding from given curie ID.

    Parameters
    --------------------
    curie_id: int
        The curie ID to map to a chunk.
    version: str
        The version of the embedding to retrieve.
    downloads_directory: str
        The directory where to store the downloads.
    """
    chunk_id = get_chunk_id_from_curie_id(curie_id, version)
    return f"{downloads_directory}/{version}/{chunk_id}.npy"


def download_chunks_from_curie_ids(
    curie_ids: np.ndarray,
    version: str,
    downloads_directory: str
):
    """Downloads embedding chunks for provided curie IDs.

    Parameters
    --------------------
    curie_ids: int
        The curie IDs to map to a chunks.
    version: str
        The version of the embedding to retrieve.
    downloads_directory: str
        The directory where to store the downloads.
    """
    urls, chunk_ids = get_unique_urls_from_curie_ids(curie_ids, version)
    BaseDownloader(
        downloads_directory=f"{downloads_directory}/{version}",
    ).download(
        urls=urls,
        paths=[f"{chunk_id}.npy" for chunk_id in chunk_ids]
    )


embeddings: Dict[str, np.ndarray] = dict()


def get_embedding_from_curie_id(
    curie_id: int,
    version: str,
    downloads_directory: str
) -> np.ndarray:
    """Return embedding chunk for provided curie ID.

    Parameters
    --------------------
    curie_id: int
        The curie ID to map to a chunk.
    version: str
        The version of the embedding to retrieve.
    downloads_directory: str
        The directory where to store the downloads.
    """
    global embeddings
    path = get_embedding_chunk_path_from_curie_id(
        curie_id, version, downloads_directory)
    if path not in embeddings:
        embeddings[path] = np.load(path, mmap_mode="r")
    return embeddings[path]


def get_vector_from_curie_id(
    curie_id: int,
    version: str,
    downloads_directory: str
) -> np.ndarray:
    """Return embedding chunk for provided curie ID.

    Parameters
    --------------------
    curie_id: int
        The curie ID to map to a chunk.
    version: str
        The version of the embedding to retrieve.
    downloads_directory: str
        The directory where to store the downloads.
    """
    return get_embedding_from_curie_id(
        curie_id,
        version,
        downloads_directory
    )[restrict_curie_id_to_chunk(curie_id, version)]
