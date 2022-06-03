pub struct Date {
    pub(crate) year: Option<u16>,
    pub(crate) month: Option<String>,
    pub(crate) day: Option<u8>,
}

pub struct JournalIssue {
    pub(crate) volume: Option<String>,
    pub(crate) issue: Option<String>,
    pub(crate) pubblication_date: Date,
}

pub struct Journal {
    pub(crate) issn: Option<String>,
    pub(crate) title: String,
    pub(crate) iso_abbreviation: Option<String>,
    pub(crate) journal_issue: JournalIssue,
}

pub struct Chemical {
    pub(crate) registry_number: String,
    pub(crate) name_of_substance: String,
    pub(crate) code: String,
}

#[derive(Debug)]
pub struct MeshTopic {
    pub(crate) name: String,
    pub(crate) code: String,
    pub(crate) is_major_topic: bool,
}

#[derive(Debug)]
pub struct Keyword {
    pub(crate) name: String,
    pub(crate) is_major_topic: bool,
}

#[derive(Debug)]
pub struct Mesh {
    pub(crate) descriptor: MeshTopic,
    pub(crate) qualifier: Option<MeshTopic>,
}

#[derive(Debug)]
pub struct SupplMesh {
    pub(crate) code: String,
    pub(crate) name: String,
    pub(crate) mesh_type: String,
}

#[derive(Debug)]
pub struct Abstract {
    pub(crate) text: String,
    pub(crate) abstract_type: Option<String>,
    pub(crate) language: Option<String>
}
#[derive(Debug)]
pub struct ArticleId {
    pub(crate) id_type: String,
    pub(crate) value: String,
}

pub struct Article {
    pub(crate) completion_date: Option<Date>,
    pub(crate) revision_date: Option<Date>,
    pub(crate) pubmed_id: u32,
    pub(crate) article_ids: Vec<ArticleId>,
    pub(crate) journal: Journal,
    pub(crate) title: Option<String>,
    pub(crate) abstract_text: Option<Abstract>,
    pub(crate) other_abstract_texts: Vec<Abstract>,
    pub(crate) chemical_list: Vec<Chemical>,
    pub(crate) gene_symbol_list: Vec<String>,
    pub(crate) mesh_list: Vec<Mesh>,
    pub(crate) suppl_mesh_list: Vec<SupplMesh>,
    pub(crate) references: Vec<usize>,
    pub(crate) keywords: Vec<Keyword>,
}
