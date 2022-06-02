use crate::article::*;
use std::{collections::HashMap, fmt::Debug, str::FromStr};

#[derive(Debug)]
struct XMLHelper {
    tag: String,
    attributes: HashMap<String, String>,
    mandatory_attributes: Option<HashMap<String, String>>,
    tag_opened: bool,
    tag_closed: bool,
}

impl XMLHelper {
    pub fn new(tag: &str) -> Self {
        XMLHelper {
            tag: tag.to_string(),
            attributes: HashMap::new(),
            mandatory_attributes: None,
            tag_opened: false,
            tag_closed: false,
        }
    }

    pub fn with_attributes(tag: &str, mandatory_attributes: HashMap<String, String>) -> Self {
        if mandatory_attributes.is_empty() {
            panic!(concat!(
                "It does not make sense to provide an empty ",
                "hashmap of mandatory attributes."
            ));
        }
        XMLHelper {
            tag: tag.to_string(),
            attributes: HashMap::new(),
            mandatory_attributes: Some(mandatory_attributes),
            tag_opened: false,
            tag_closed: false,
        }
    }

    pub fn can_build(&self) -> bool {
        self.tag_closed
    }

    pub fn parse<'a>(&'a mut self, line: &'a str) -> Result<&'a str, String> {
        let opening_tag = format!("<{}", self.tag);
        let line = if line.starts_with(&opening_tag)
            && [">", " "].contains(&&line[opening_tag.len()..opening_tag.len() + 1])
        {
            if self.tag_opened {
                return Err(format!(
                    "Tag {} is already opened! Reading the line {}.",
                    self.tag, line
                ));
            }
            let tag_length = line.find(">").unwrap();
            let attributes_portion = &line[opening_tag.len()..tag_length];

            let attributes = attributes_portion
                .trim()
                .split(" ")
                .filter_map(|attribute| {
                    if attribute.is_empty() || !attribute.contains('=') {
                        return None;
                    }
                    let key_and_value = attribute
                        .split("=")
                        .map(|a| a.to_string())
                        .collect::<Vec<String>>();
                    if key_and_value.len() < 2 {
                        panic!("{:?}, {}", key_and_value, line);
                    }
                    let first = key_and_value[0].clone();
                    let second = key_and_value[1].clone();
                    Some((first, second.trim_matches('\"').to_string()))
                })
                .collect::<HashMap<String, String>>();

            if let Some(mandatory_attributes) = self.mandatory_attributes.as_ref() {
                if !mandatory_attributes
                    .iter()
                    .all(|(key, value)| match attributes.get(key) {
                        Some(current_value) => current_value == value,
                        None => false,
                    })
                {
                    return Ok("");
                }
            }

            self.attributes = attributes;
            self.tag_opened = true;

            &line[tag_length + 1..]
        } else {
            line
        };

        if !self.tag_opened {
            return Ok("");
        }

        let closing_tag = format!("</{}>", self.tag);

        let line = if line.ends_with(&closing_tag) {
            if self.tag_closed {
                return Err(format!("Tag {} is already closed!", self.tag));
            } else if !self.tag_opened {
                return Err(format!(
                    "Trying to close tag {} when it was not yet opened!",
                    self.tag
                ));
            }
            self.tag_closed = true;
            &line[..line.len() - closing_tag.len()]
        } else {
            line
        };

        Ok(line)
    }
}

#[derive(Debug)]
struct ObjectBuilder<T: FromStr + Debug> {
    xml_helper: XMLHelper,
    value: Option<T>,
}

impl<T: FromStr + Debug> ObjectBuilder<T> {
    pub fn new(tag: &str) -> Self {
        ObjectBuilder {
            xml_helper: XMLHelper::new(tag),
            value: None,
        }
    }

    pub fn with_attributes(tag: &str, mandatory_attributes: HashMap<String, String>) -> Self {
        ObjectBuilder {
            xml_helper: XMLHelper::with_attributes(tag, mandatory_attributes),
            value: None,
        }
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if !line.is_empty() {
            if self.value.is_some() {
                let line = line.to_string();
                let tag = self.xml_helper.tag.clone();
                return Err(format!(
                    concat!(
                        "The parser for the tag {} ",
                        "has already a value {:?} ",
                        "but a new value {} was now ",
                        "provided."
                    ),
                    tag, self.value, line
                ));
            } else {
                self.value = Some(T::from_str(line).map_err(|_| {
                    format!(
                        concat!("Something went wrong while trying to convert the value `{}`."),
                        line
                    )
                })?);
            }
            return Ok(true);
        }
        Ok(self.xml_helper.tag_opened)
    }

    pub fn build(self) -> Option<T> {
        self.value
    }
}

#[derive(Debug)]
struct DateBuilder {
    xml_helper: XMLHelper,
    year_builder: ObjectBuilder<u16>,
    month_builder: ObjectBuilder<String>,
    day_builder: ObjectBuilder<u8>,
}

impl DateBuilder {
    pub fn new(tag: &str) -> Self {
        DateBuilder {
            xml_helper: XMLHelper::new(tag),
            year_builder: ObjectBuilder::new("Year"),
            month_builder: ObjectBuilder::new("Month"),
            day_builder: ObjectBuilder::new("Day"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if !self.year_builder.can_build() && self.year_builder.parse(line)? {
            return Ok(true);
        }
        if !self.month_builder.can_build() && self.month_builder.parse(line)? {
            return Ok(true);
        }
        if !self.day_builder.can_build() && self.day_builder.parse(line)? {
            return Ok(true);
        }
        Ok(self.xml_helper.tag_opened)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }

    pub fn build(self) -> Result<Date, String> {
        if !self.xml_helper.can_build() {
            return Err(format!(
                concat!(
                    "Build method was called on DateBuilder ",
                    "but the object is not yet ready to build. ",
                    "The current status is: {:?}."
                ),
                self
            ));
        }
        Ok(Date {
            year: self.year_builder.build(),
            month: self.month_builder.build(),
            day: self.day_builder.build(),
        })
    }
}

struct JournalIssueBuilder {
    xml_helper: XMLHelper,
    volume_builder: ObjectBuilder<String>,
    issue_builder: ObjectBuilder<String>,
    pubblication_date_builder: DateBuilder,
}

impl JournalIssueBuilder {
    pub fn new() -> Self {
        JournalIssueBuilder {
            xml_helper: XMLHelper::new("JournalIssue"),
            volume_builder: ObjectBuilder::new("Volume"),
            issue_builder: ObjectBuilder::new("Issue"),
            pubblication_date_builder: DateBuilder::new("PubDate"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if !self.volume_builder.can_build() && self.volume_builder.parse(line)? {
            return Ok(true);
        }
        if !self.issue_builder.can_build() && self.issue_builder.parse(line)? {
            return Ok(true);
        }
        if !self.pubblication_date_builder.can_build()
            && self.pubblication_date_builder.parse(line)?
        {
            return Ok(true);
        }
        Ok(self.xml_helper.tag_opened)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }

    pub fn build(self) -> Result<JournalIssue, String> {
        if !self.xml_helper.can_build() {
            return Err(format!(concat!(
                "Build method was called on JournalBuilder ",
                "but the object is not yet ready to build."
            )));
        }
        Ok(JournalIssue {
            volume: self.volume_builder.build(),
            issue: self.issue_builder.build(),
            pubblication_date: self.pubblication_date_builder.build()?,
        })
    }
}

struct JournalBuilder {
    xml_helper: XMLHelper,
    issn_builder: ObjectBuilder<String>,
    title_builder: ObjectBuilder<String>,
    iso_abbreviation_builder: ObjectBuilder<String>,
    journal_issue_builder: JournalIssueBuilder,
}

impl JournalBuilder {
    pub fn new() -> Self {
        JournalBuilder {
            xml_helper: XMLHelper::new("Journal"),
            issn_builder: ObjectBuilder::new("ISSN"),
            title_builder: ObjectBuilder::new("Title"),
            iso_abbreviation_builder: ObjectBuilder::new("ISOAbbreviation"),
            journal_issue_builder: JournalIssueBuilder::new(),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if line.is_empty() {
            return Ok(self.xml_helper.tag_opened);
        }
        if !self.issn_builder.can_build() && self.issn_builder.parse(line)? {
            return Ok(true);
        }
        if !self.journal_issue_builder.can_build() && self.journal_issue_builder.parse(line)? {
            return Ok(true);
        }
        if !self.title_builder.can_build() && self.title_builder.parse(line)? {
            return Ok(true);
        }
        if !self.iso_abbreviation_builder.can_build()
            && self.iso_abbreviation_builder.parse(line)?
        {
            return Ok(true);
        }
        Ok(self.xml_helper.tag_opened)
    }

    pub fn build(self) -> Result<Journal, String> {
        if !self.xml_helper.can_build() {
            return Err(format!(concat!(
                "Build method was called on JournalBuilder ",
                "but the object is not yet ready to build."
            )));
        }
        Ok(Journal {
            issn: self.issn_builder.build(),
            title: self.title_builder.build().unwrap(),
            iso_abbreviation: self.iso_abbreviation_builder.build().unwrap(),
            journal_issue: self.journal_issue_builder.build()?,
        })
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

struct AbstractBuilder {
    xml_helper: XMLHelper,
    abstract_builder: ObjectBuilder<String>,
}

impl AbstractBuilder {
    pub fn new(tag: &str) -> Self {
        AbstractBuilder {
            xml_helper: XMLHelper::new(tag),
            abstract_builder: ObjectBuilder::new("AbstractText"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if line.is_empty() {
            return Ok(self.xml_helper.tag_opened);
        }
        if !self.abstract_builder.can_build() && self.abstract_builder.parse(line)? {
            return Ok(true);
        }
        Ok(self.xml_helper.tag_opened)
    }

    pub fn build(self) -> Result<String, String> {
        if !self.xml_helper.can_build() {
            return Err(format!(concat!(
                "Build method was called on AbstractBuilder ",
                "but the object is not yet ready to build."
            )));
        }
        Ok(self.abstract_builder.build().unwrap())
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

struct ChemicalBuilder {
    xml_helper: XMLHelper,
    registry_number_builder: ObjectBuilder<String>,
    name_of_substance_builder: ObjectBuilder<String>,
}

impl ChemicalBuilder {
    pub fn new() -> Self {
        ChemicalBuilder {
            xml_helper: XMLHelper::new("Chemical"),
            registry_number_builder: ObjectBuilder::new("RegistryNumber"),
            name_of_substance_builder: ObjectBuilder::new("NameOfSubstance"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if !self.registry_number_builder.can_build() && self.registry_number_builder.parse(line)? {
            return Ok(true);
        }
        if !self.name_of_substance_builder.can_build()
            && self.name_of_substance_builder.parse(line)?
        {
            return Ok(true);
        }
        Ok(self.xml_helper.tag_opened)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }

    pub fn build(self) -> Result<Chemical, String> {
        Ok(Chemical {
            registry_number: self.registry_number_builder.build().unwrap(),
            code: self
                .name_of_substance_builder
                .xml_helper
                .attributes
                .get("UI")
                .unwrap()
                .clone(),
            name_of_substance: self.name_of_substance_builder.build().unwrap(),
        })
    }
}

struct ChemicalListBuilder {
    xml_helper: XMLHelper,
    chemicals: Vec<Chemical>,
    chemical_builder: ChemicalBuilder,
}

impl ChemicalListBuilder {
    pub fn new() -> Self {
        ChemicalListBuilder {
            xml_helper: XMLHelper::new("ChemicalList"),
            chemicals: Vec::new(),
            chemical_builder: ChemicalBuilder::new(),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if line.is_empty() {
            return Ok(self.xml_helper.tag_opened);
        }

        self.chemical_builder.parse(line)?;
        if self.chemical_builder.can_build() {
            self.chemicals.push(
                core::mem::replace(&mut self.chemical_builder, ChemicalBuilder::new())
                    .build()
                    .unwrap(),
            );
        }

        Ok(self.xml_helper.tag_opened)
    }

    pub fn build(self) -> Result<Vec<Chemical>, String> {
        if !self.xml_helper.can_build() && !self.chemicals.is_empty() {
            return Err(format!(concat!(
                "Build method was called on ChemicalListBuilder ",
                "but the object is not yet ready to build."
            )));
        }
        Ok(self.chemicals)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

#[derive(Debug)]
struct MeshBuilder {
    xml_helper: XMLHelper,
    descriptor_builder: ObjectBuilder<String>,
    qualifier_builder: ObjectBuilder<String>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        MeshBuilder {
            xml_helper: XMLHelper::new("MeshHeading"),
            descriptor_builder: ObjectBuilder::new("DescriptorName"),
            qualifier_builder: ObjectBuilder::new("QualifierName"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if !self.descriptor_builder.can_build() && self.descriptor_builder.parse(line)? {
            return Ok(true);
        }
        if !self.qualifier_builder.can_build() && self.qualifier_builder.parse(line)? {
            return Ok(true);
        }
        Ok(self.xml_helper.tag_opened)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }

    pub fn build(self) -> Result<Mesh, String> {
        let descriptor = MeshTopic {
            code: self
                .descriptor_builder
                .xml_helper
                .attributes
                .get("UI")
                .unwrap()
                .clone(),
            is_major_topic: self
                .descriptor_builder
                .xml_helper
                .attributes
                .get("MajorTopicYN")
                .unwrap()
                .clone()
                == "Y",
            name: self.descriptor_builder.build().unwrap(),
        };

        let qualifier = if self.qualifier_builder.can_build() {
            Some(MeshTopic {
                code: self
                    .qualifier_builder
                    .xml_helper
                    .attributes
                    .get("UI")
                    .unwrap()
                    .clone(),
                is_major_topic: self
                    .qualifier_builder
                    .xml_helper
                    .attributes
                    .get("MajorTopicYN")
                    .unwrap()
                    .clone()
                    == "Y",
                name: self.qualifier_builder.build().unwrap(),
            })
        } else {
            None
        };

        Ok(Mesh {
            descriptor,
            qualifier,
        })
    }
}

#[derive(Debug)]
struct MeshListBuilder {
    xml_helper: XMLHelper,
    meshes: Vec<Mesh>,
    mesh_builder: MeshBuilder,
}

impl MeshListBuilder {
    pub fn new() -> Self {
        MeshListBuilder {
            xml_helper: XMLHelper::new("MeshHeadingList"),
            meshes: Vec::new(),
            mesh_builder: MeshBuilder::new(),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if line.is_empty() {
            return Ok(self.xml_helper.tag_opened);
        }
        self.mesh_builder.parse(line)?;
        if self.mesh_builder.can_build() {
            self.meshes.push(
                core::mem::replace(&mut self.mesh_builder, MeshBuilder::new())
                    .build()
                    .unwrap(),
            );
        }

        Ok(self.xml_helper.tag_opened)
    }

    pub fn build(self) -> Result<Vec<Mesh>, String> {
        if !self.xml_helper.can_build() {
            return Err(format!(
                concat!(
                    "Build method was called on MeshListBuilder ",
                    "but the object is not yet ready to build. ",
                    "The object currently looks like {:?}"
                ),
                self
            ));
        }
        Ok(self.meshes)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

#[derive(Debug)]
struct KeywordListBuilder {
    xml_helper: XMLHelper,
    keywords: Vec<Keyword>,
    keyword_builder: ObjectBuilder<String>,
}

impl KeywordListBuilder {
    pub fn new() -> Self {
        KeywordListBuilder {
            xml_helper: XMLHelper::new("KeywordList"),
            keywords: Vec::new(),
            keyword_builder: ObjectBuilder::new("Keyword"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if line.is_empty() {
            return Ok(self.xml_helper.tag_opened);
        }
        self.keyword_builder.parse(line)?;
        if self.keyword_builder.can_build() {
            let keyword_builder =
                core::mem::replace(&mut self.keyword_builder, ObjectBuilder::new("Keyword"));
            self.keywords.push(Keyword {
                is_major_topic: keyword_builder
                    .xml_helper
                    .attributes
                    .get("MajorTopicYN")
                    .unwrap()
                    .clone()
                    == "Y",
                name: keyword_builder.build().unwrap(),
            })
        }

        Ok(self.xml_helper.tag_opened)
    }

    pub fn build(self) -> Result<Vec<Keyword>, String> {
        if !self.xml_helper.can_build() && !self.keywords.is_empty() {
            return Err(format!(
                concat!(
                    "Build method was called on KeywordListBuilder ",
                    "but the object is not yet ready to build. ",
                    "The object currently looks like {:?}"
                ),
                self
            ));
        }
        Ok(self.keywords)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

#[derive(Debug)]
struct ReferencesBuilder {
    xml_helper: XMLHelper,
    references: Vec<usize>,
    pubmed_builder: ObjectBuilder<usize>,
}

impl ReferencesBuilder {
    pub fn new() -> Self {
        ReferencesBuilder {
            xml_helper: XMLHelper::new("ReferenceList"),
            references: Vec::new(),
            pubmed_builder: ObjectBuilder::with_attributes(
                "ArticleId",
                [("IdType".to_string(), "pubmed".to_string())]
                    .into_iter()
                    .collect(),
            ),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if line.is_empty() {
            return Ok(self.xml_helper.tag_opened);
        }
        self.pubmed_builder.parse(line)?;
        if self.pubmed_builder.can_build() {
            self.references.push(
                core::mem::replace(
                    &mut self.pubmed_builder,
                    ObjectBuilder::with_attributes(
                        "ArticleId",
                        [("IdType".to_string(), "pubmed".to_string())]
                            .into_iter()
                            .collect(),
                    ),
                )
                .build()
                .unwrap(),
            );
        }

        Ok(self.xml_helper.tag_opened)
    }

    pub fn build(self) -> Result<Vec<usize>, String> {
        if !self.xml_helper.can_build() && !self.references.is_empty() {
            return Err(format!(
                concat!(
                    "Build method was called on ReferencesBuilder ",
                    "but the object is not yet ready to build. ",
                    "The object currently looks like {:?}"
                ),
                self
            ));
        }
        Ok(self.references)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

struct AuthorListBuilder {
    xml_helper: XMLHelper,
}

impl AuthorListBuilder {
    pub fn new() -> Self {
        AuthorListBuilder {
            xml_helper: XMLHelper::new("AuthorList"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let _ = self.xml_helper.parse(line)?;
        Ok(self.xml_helper.tag_opened)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

struct PublicationTypeListBuilder {
    xml_helper: XMLHelper,
}

impl PublicationTypeListBuilder {
    pub fn new() -> Self {
        PublicationTypeListBuilder {
            xml_helper: XMLHelper::new("PublicationTypeList"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let _ = self.xml_helper.parse(line)?;
        Ok(self.xml_helper.tag_opened)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

struct MedlineJournalInfoBuilder {
    xml_helper: XMLHelper,
}

impl MedlineJournalInfoBuilder {
    pub fn new() -> Self {
        MedlineJournalInfoBuilder {
            xml_helper: XMLHelper::new("MedlineJournalInfo"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let _ = self.xml_helper.parse(line)?;
        Ok(self.xml_helper.tag_opened)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

struct HistoryBuilder {
    xml_helper: XMLHelper,
}

impl HistoryBuilder {
    pub fn new() -> Self {
        HistoryBuilder {
            xml_helper: XMLHelper::new("History"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let _ = self.xml_helper.parse(line)?;
        Ok(self.xml_helper.tag_opened)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

struct PersonalNameSubjectListBuilder {
    xml_helper: XMLHelper,
}

impl PersonalNameSubjectListBuilder {
    pub fn new() -> Self {
        PersonalNameSubjectListBuilder {
            xml_helper: XMLHelper::new("PersonalNameSubjectList"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let _ = self.xml_helper.parse(line)?;
        Ok(self.xml_helper.tag_opened)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

struct DataBankListBuilder {
    xml_helper: XMLHelper,
}

impl DataBankListBuilder {
    pub fn new() -> Self {
        DataBankListBuilder {
            xml_helper: XMLHelper::new("DataBankList"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let _ = self.xml_helper.parse(line)?;
        Ok(self.xml_helper.tag_opened)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

struct GrantListBuilder {
    xml_helper: XMLHelper,
}

impl GrantListBuilder {
    pub fn new() -> Self {
        GrantListBuilder {
            xml_helper: XMLHelper::new("GrantList"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let _ = self.xml_helper.parse(line)?;
        Ok(self.xml_helper.tag_opened)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

struct CommentsCorrectionsListBuilder {
    xml_helper: XMLHelper,
}

impl CommentsCorrectionsListBuilder {
    pub fn new() -> Self {
        CommentsCorrectionsListBuilder {
            xml_helper: XMLHelper::new("CommentsCorrectionsList"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let _ = self.xml_helper.parse(line)?;
        Ok(self.xml_helper.tag_opened)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

pub(crate) struct ArticleBuilder {
    xml_helper: XMLHelper,
    completion_date_builder: DateBuilder,
    revised_date_builder: DateBuilder,
    pmid_builder: ObjectBuilder<u32>,
    pubmed_builder: ObjectBuilder<String>,
    doi_builder: ObjectBuilder<String>,
    pii_builder: ObjectBuilder<String>,
    mid_builder: ObjectBuilder<String>,
    pmc_builder: ObjectBuilder<String>,
    journal_builder: JournalBuilder,
    title_builder: ObjectBuilder<String>,
    abstract_builder: AbstractBuilder,
    other_abstract_builder: AbstractBuilder,
    author_list_builder: AuthorListBuilder,
    publication_type_list_builder: PublicationTypeListBuilder,
    language_builder: ObjectBuilder<String>,
    medline_journal_info_builder: MedlineJournalInfoBuilder,
    history_builder: HistoryBuilder,
    data_bank_builder: DataBankListBuilder,
    grant_builder: GrantListBuilder,
    comments_corrections_builder: CommentsCorrectionsListBuilder,
    personal_name_subject_list: PersonalNameSubjectListBuilder,
    chemical_list_builder: ChemicalListBuilder,
    mesh_list_builder: MeshListBuilder,
    references_builder: ReferencesBuilder,
    keywords_builder: KeywordListBuilder,
}

impl ArticleBuilder {
    pub fn new() -> Self {
        ArticleBuilder {
            xml_helper: XMLHelper::new("PubmedArticle"),
            completion_date_builder: DateBuilder::new("DateCompleted"),
            revised_date_builder: DateBuilder::new("DateRevised"),
            pmid_builder: ObjectBuilder::new("PMID"),
            pubmed_builder: ObjectBuilder::with_attributes(
                "ArticleId",
                [("IdType".to_string(), "pubmed".to_string())]
                    .into_iter()
                    .collect(),
            ),
            doi_builder: ObjectBuilder::with_attributes(
                "ArticleId",
                [("IdType".to_string(), "doi".to_string())]
                    .into_iter()
                    .collect(),
            ),
            pii_builder: ObjectBuilder::with_attributes(
                "ArticleId",
                [("IdType".to_string(), "pii".to_string())]
                    .into_iter()
                    .collect(),
            ),
            mid_builder: ObjectBuilder::with_attributes(
                "ArticleId",
                [("IdType".to_string(), "mid".to_string())]
                    .into_iter()
                    .collect(),
            ),
            pmc_builder: ObjectBuilder::with_attributes(
                "ArticleId",
                [("IdType".to_string(), "pmc".to_string())]
                    .into_iter()
                    .collect(),
            ),
            journal_builder: JournalBuilder::new(),
            title_builder: ObjectBuilder::new("ArticleTitle"),
            abstract_builder: AbstractBuilder::new("Abstract"),
            other_abstract_builder: AbstractBuilder::new("OtherAbstract"),
            author_list_builder: AuthorListBuilder::new(),
            language_builder: ObjectBuilder::new("Language"),
            publication_type_list_builder: PublicationTypeListBuilder::new(),
            medline_journal_info_builder: MedlineJournalInfoBuilder::new(),
            history_builder: HistoryBuilder::new(),
            data_bank_builder: DataBankListBuilder::new(),
            grant_builder: GrantListBuilder::new(),
            comments_corrections_builder: CommentsCorrectionsListBuilder::new(),
            personal_name_subject_list: PersonalNameSubjectListBuilder::new(),
            chemical_list_builder: ChemicalListBuilder::new(),
            mesh_list_builder: MeshListBuilder::new(),
            references_builder: ReferencesBuilder::new(),
            keywords_builder: KeywordListBuilder::new(),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<(), String> {
        let line = self.xml_helper.parse(line)?;
        if line.is_empty() {
            return Ok(());
        }
        if !self.completion_date_builder.can_build() && self.completion_date_builder.parse(line)? {
            return Ok(());
        }
        if !self.revised_date_builder.can_build() && self.revised_date_builder.parse(line)? {
            return Ok(());
        }
        if !self.pmid_builder.can_build() && self.pmid_builder.parse(line)? {
            return Ok(());
        }
        if !self.pubmed_builder.can_build() && self.pubmed_builder.parse(line)? {
            return Ok(());
        }
        if !self.doi_builder.can_build() && self.doi_builder.parse(line)? {
            return Ok(());
        }
        if !self.pii_builder.can_build() && self.pii_builder.parse(line)? {
            return Ok(());
        }
        if !self.mid_builder.can_build() && self.mid_builder.parse(line)? {
            return Ok(());
        }
        if !self.pmc_builder.can_build() && self.pmc_builder.parse(line)? {
            return Ok(());
        }
        if !self.journal_builder.can_build() && self.journal_builder.parse(line)? {
            return Ok(());
        }
        if !self.title_builder.can_build() && self.title_builder.parse(line)? {
            return Ok(());
        }
        if !self.abstract_builder.can_build() && self.abstract_builder.parse(line)? {
            return Ok(());
        }
        if !self.other_abstract_builder.can_build() && self.other_abstract_builder.parse(line)? {
            return Ok(());
        }
        if !self.author_list_builder.can_build() && self.author_list_builder.parse(line)? {
            return Ok(());
        }
        if self.language_builder.parse(line)? {
            return Ok(());
        }
        if !self.publication_type_list_builder.can_build()
            && self.publication_type_list_builder.parse(line)?
        {
            return Ok(());
        }
        if !self.medline_journal_info_builder.can_build()
            && self.medline_journal_info_builder.parse(line)?
        {
            return Ok(());
        }
        if !self.chemical_list_builder.can_build() && self.chemical_list_builder.parse(line)? {
            return Ok(());
        }
        if !self.mesh_list_builder.can_build() && self.mesh_list_builder.parse(line)? {
            return Ok(());
        }
        if !self.keywords_builder.can_build() && self.keywords_builder.parse(line)? {
            return Ok(());
        }
        if !self.references_builder.can_build() && self.references_builder.parse(line)? {
            return Ok(());
        }
        if !self.history_builder.can_build() && self.history_builder.parse(line)? {
            return Ok(());
        }
        if !self.data_bank_builder.can_build() && self.data_bank_builder.parse(line)? {
            return Ok(());
        }
        if !self.grant_builder.can_build() && self.grant_builder.parse(line)? {
            return Ok(());
        }
        if !self.comments_corrections_builder.can_build() && self.comments_corrections_builder.parse(line)? {
            return Ok(());
        }
        if !self.personal_name_subject_list.can_build()
            && self.personal_name_subject_list.parse(line)?
        {
            return Ok(());
        }

        Err(format!("The line {} was not handled by any parser.", line))
    }

    pub fn build(self) -> Result<Article, String> {
        if !self.xml_helper.can_build() {
            return Err("The article is not ready!".to_string());
        }
        Ok(Article {
            completion_date: self.completion_date_builder.build().unwrap(),
            revision_date: self.revised_date_builder.build().ok(),
            pubmed_id: self.pmid_builder.build().unwrap(),
            doi: self.doi_builder.build(),
            pii: self.pii_builder.build(),
            mid: self.mid_builder.build(),
            pmc: self.pmc_builder.build(),
            journal: self.journal_builder.build()?,
            title: self.title_builder.build().unwrap(),
            abstract_text: self.abstract_builder.build().ok(),
            other_abstract_text: self.other_abstract_builder.build().ok(),
            chemical_list: self.chemical_list_builder.build()?,
            mesh_list: self.mesh_list_builder.build()?,
            references: self.references_builder.build()?,
            keywords: self.keywords_builder.build()?,
        })
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}