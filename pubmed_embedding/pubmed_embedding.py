from typing import Union, List
from tqdm.auto import tqdm
import pandas as pd
import numpy as np
from .utils import get_index, download_chunks_from_curie_ids, get_vector_from_curie_id


def get_pubmed_embedding_from_curies(
    curies: Union[List[str], str],
    downloads_directory: str = "embeddings",
    version: str = "pubmed_scibert_26_11_2022",
) -> pd.DataFrame:
    """Returns dataframe with curies as index and embedding from required version.
    
    Parameters
    ---------------------
    curies: Union[List[str], str]
        Curies to retrieve the embedding for.
    downloads_directory: str = "embeddings"
        Directory where to store the downloaded files.
    version: str = "pubmed_scibert_26_11_2022"
        The version of the file to retrieve.
    """
    index: pd.DataFrame = get_index(
        version=version,
        downloads_directory=downloads_directory
    )
    
    if isinstance(curies, str):
        curies: List[str] = [curies]
    
    curie_ids: np.ndarray = index.loc[curies].index.values

    download_chunks_from_curie_ids(curie_ids, version, downloads_directory)

    return pd.DataFrame(
        np.array([
            get_vector_from_curie_id(
                curie_id=curie_id,
                version=version,
                downloads_directory=downloads_directory
            )
            for curie_id in tqdm(
                curie_ids,
                desc=f"Retrieving embedding version {version}",
                leave=False,
                dynamic_ncols=True,
                disable=len(curie_ids) == 1
            )
        ]),
        index=curies
    )