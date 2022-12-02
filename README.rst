PubMed embedding
===================================
Building PubMed embedding, automatically.


Install the package
----------------------------------
As usual, just install from Pypi:

.. code:: shell

    pip install pubmed_embedding


Usage examples
----------------------------------
You can retrieve embedding for PubMed IDs of interest as such:

BERT
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code:: python

    from pubmed_embedding import get_pubmed_embedding_from_curies

    pubmed_ids = ["PMID:24774509", "PMID:15170967", "PMID:7850793"]

    get_pubmed_embedding_from_curies(
        curies=pubmed_ids,
        version="pubmed_bert_30_11_2022"
    )

SciBERT
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code:: python

    get_pubmed_embedding_from_curies(
        curies=pubmed_ids,
        version="pubmed_scibert_30_11_2022"
    )

Specter
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code:: python

    get_pubmed_embedding_from_curies(
        curies=pubmed_ids,
        version="pubmed_specter_30_11_2022"
    )