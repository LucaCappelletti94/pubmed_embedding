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

Citing this work
-----------------------------
If you have found these datasets useful, please do cite:

.. code:: bib

    @software{cappellettiPubMed2022,
        author = {Luca, Cappelletti and Tommaso, Fontana and Justin, Reese},
        month = {12},
        title = {{BM25-weighted BERT-based embedding of PubMed}},
        url = {https://github.com/LucaCappelletti94/pubmed_embedding},
        version = {1.0.12},
        year = {2022}
    }



.. |BERT| image:: https://github.com/LucaCappelletti94/pubmed_embedding/blob/main/bert.png?raw=true
.. |SciBERT| image:: https://github.com/LucaCappelletti94/pubmed_embedding/blob/main/scibert.png?raw=true
.. |Specter| image:: https://github.com/LucaCappelletti94/pubmed_embedding/blob/main/specter.png?raw=true

