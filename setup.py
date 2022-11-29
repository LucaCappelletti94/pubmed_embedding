"""Module installing the Embiggen package."""
from setuptools import find_packages, setup
from codecs import open as copen
import os
import re

here = os.path.abspath(os.path.dirname(__file__))


def readme():
    with open('README.rst', encoding="utf8") as f:
        return f.read()


def read(*parts):
    with copen(os.path.join(here, *parts), 'r', encoding="utf8") as fp:
        return fp.read()


def find_version(*file_paths):
    version_file = read(*file_paths)
    version_match = re.search(r"^__version__ = ['\"]([^'\"]*)['\"]",
                              version_file, re.M)
    if version_match:
        return version_match.group(1)
    raise RuntimeError("Unable to find version string.")


__version__ = find_version("pubmed_embedding", "__version__.py")

setup(
    name='pubmed_embedding',
    version=__version__,
    description='A tool to work with pre-computed large pubmed embedding.',
    long_description=readme(),
    url='https://github.com/LucaCappelletti94/pubmed_graph',
    keywords='SciBERT,PubMed,BM25',
    author="Luca Cappelletti",
    license='BSD3',
    python_requires='>=3.6.0',
    packages=find_packages(
        exclude=['contrib', 'docs', 'tests*', 'notebooks*']),
    install_requires=[
        'numpy',
        'pandas',
        "tqdm",
        "compress_json",
        "downloaders",
        "userinput"
    ],
    include_package_data=True,
)
