from typing import Union, List
from tqdm.auto import tqdm
import pandas as pd
import numpy as np
from .utils import get_index, download_chunks_from_curie_ids, get_vector_from_curie_id


def get_pubmed_embedding_from_curies(
    curies: Union[List[str], str],
    ignore_missing_curies: bool = True,
    check_for_prefix: bool = True,
    downloads_directory: str = "embeddings",
    version: str = "pubmed_scibert_30_11_2022",
) -> pd.DataFrame:
    """Returns dataframe with curies as index and embedding from required version.
    
    Parameters
    ---------------------
    curies: Union[List[str], str]
        Curies to retrieve the embedding for.
    ignore_missing_curies: bool = True
        Whether to ignore curies for which we cannot currently
        provide an embedding. By default, True.
    check_for_prefix: bool = True
        Whether to check for the presence of the prefix `PMID`.
        By default True.
    downloads_directory: str = "embeddings"
        Directory where to store the downloaded files.
    version: str = "pubmed_scibert_30_11_2022"
        The version of the file to retrieve.
    """
    index: pd.DataFrame = get_index(
        version=version,
        downloads_directory=downloads_directory
    )
    
    if isinstance(curies, str):
        curies: List[str] = [curies]

    # Checking presence of prefix
    if check_for_prefix:
        for curie in curies:
            if not curie.startswith("PMID:"):
                raise ValueError(
                    f"The provided curie `{curie}` does not "
                    "begin with the expected curie prefix `PMID:`."
                )

    if ignore_missing_curies:
        curies = [
            curie
            for curie in tqdm(
                curies,
                desc=f"Checking availability in version {version}",
                leave=False,
                dynamic_ncols=True,
                disable=len(curies) == 1
            )
            if curie in index.index
        ]
    
    curie_ids: np.ndarray = index.loc[curies].curie_id.values

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