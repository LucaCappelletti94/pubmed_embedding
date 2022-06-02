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
    pub(crate) iso_abbreviation: String,
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

pub struct Article {
    pub(crate) completion_date: Date,
    pub(crate) revision_date: Option<Date>,
    pub(crate) pubmed_id: u32,
    pub(crate) doi: Option<String>,
    pub(crate) pii: Option<String>,
    pub(crate) mid: Option<String>,
    pub(crate) pmc: Option<String>,
    pub(crate) journal: Journal,
    pub(crate) title: String,
    pub(crate) abstract_text: Option<String>,
    pub(crate) other_abstract_text: Option<String>,
    pub(crate) chemical_list: Vec<Chemical>,
    pub(crate) mesh_list: Vec<Mesh>,
    pub(crate) references: Vec<usize>,
    pub(crate) keywords: Vec<Keyword>
}