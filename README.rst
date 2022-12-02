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

    bert_features = get_pubmed_embedding_from_curies(
        curies=pubmed_ids,
        version="pubmed_bert_30_11_2022"
    )

And the result is:

|BERT|


SciBERT
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code:: python

    scibert_features = get_pubmed_embedding_from_curies(
        curies=pubmed_ids,
        version="pubmed_scibert_30_11_2022"
    )
   
And the result is:

|SciBERT|

Specter
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code:: python

    spected_features = get_pubmed_embedding_from_curies(
        curies=pubmed_ids,
        version="pubmed_specter_30_11_2022"
    )

And the result is:

|Specter|


.. |BERT| image:: https://github.com/LucaCappelletti94/pubmed_embedding/blob/main/bert.png?raw=true
.. |SciBERT| image:: https://github.com/LucaCappelletti94/pubmed_embedding/blob/main/scibert.png?raw=true
.. |Specter| image:: https://github.com/LucaCappelletti94/pubmed_embedding/blob/main/specter.png?raw=true
