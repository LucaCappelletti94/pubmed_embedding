use crate::article::*;
use std::{collections::HashMap, fmt::Debug, str::FromStr};

#[derive(Debug)]
struct XMLHelper {
    tag: String,
    attributes: HashMap<String, String>,
    mandatory_attributes: Option<HashMap<String, String>>,
    tag_opened: bool,
    just_opened: bool,
    tag_closed: bool,
    just_closed: bool,
    allow_reopening: bool,
    openings: u8,
}

impl XMLHelper {
    pub fn new(tag: &str) -> Self {
        XMLHelper {
            tag: tag.to_string(),
            attributes: HashMap::new(),
            mandatory_attributes: None,
            tag_opened: false,
            just_opened: false,
            tag_closed: false,
            just_closed: false,
            allow_reopening: false,
            openings: 0,
        }
    }

    pub fn with_reopening(tag: &str) -> Self {
        XMLHelper {
            tag: tag.to_string(),
            attributes: HashMap::new(),
            mandatory_attributes: None,
            tag_opened: false,
            just_opened: false,
            tag_closed: false,
            just_closed: false,
            allow_reopening: true,
            openings: 0,
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
            just_opened: false,
            tag_closed: false,
            just_closed: false,
            allow_reopening: false,
            openings: 0,
        }
    }

    pub fn can_build(&self) -> bool {
        self.tag_closed && self.openings == 0
    }

    pub fn parse<'a>(&'a mut self, line: &'a str) -> Result<&'a str, String> {
        let opening_tag = format!("<{}", self.tag);
        let line = if line.starts_with(&opening_tag)
            && [">", " "].contains(&&line[opening_tag.len()..opening_tag.len() + 1])
        {
            if !self.allow_reopening && self.tag_opened && !self.just_opened {
                return Err(format!(
                    "Tag {} is already opened! Reading the line {}.",
                    self.tag, line
                ));
            }
            if line.ends_with("/>") {
                return Ok("");
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
            self.tag_closed = false;
            self.just_closed = false;
            self.openings += 1;
            self.just_opened = true;

            &line[tag_length + 1..]
        } else {
            self.just_opened = false;
            line
        };

        if !self.tag_opened {
            return Ok("");
        }

        let closing_tag = format!("</{}>", self.tag);

        let line = if line.ends_with(&closing_tag) {
            if self.tag_closed && !self.just_closed && !self.allow_reopening {
                return Err(format!("Tag {} is already closed!", self.tag));
            } else if !self.tag_opened {
                return Err(format!(
                    "Trying to close tag {} when it was not yet opened!",
                    self.tag
                ));
            }
            self.tag_closed = true;
            self.just_closed = true;
            self.openings -= 1;

            &line[..line.len() - closing_tag.len()]
        } else {
            self.just_closed = false;
            line
        };

        Ok(line)
    }
}

#[derive(Debug)]
struct ObjectBuilder<T: FromStr + Debug> {
    xml_helper: XMLHelper,
    textual_value: String,
    value: Option<T>,
}

impl<T: FromStr + Debug> ObjectBuilder<T> {
    pub fn new(tag: &str) -> Self {
        ObjectBuilder {
            xml_helper: XMLHelper::new(tag),
            textual_value: "".to_string(),
            value: None,
        }
    }

    pub fn with_attributes(tag: &str, mandatory_attributes: HashMap<String, String>) -> Self {
        ObjectBuilder {
            xml_helper: XMLHelper::with_attributes(tag, mandatory_attributes),
            textual_value: "".to_string(),
            value: None,
        }
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if !line.is_empty() {
            self.textual_value = if self.textual_value.is_empty() {
                line.to_string()
            } else {
                format!("{} {}", self.textual_value, line)
            };
        }
        if self.can_build() {
            self.value = Some(T::from_str(self.textual_value.trim()).map_err(|_| {
                format!(
                    concat!(
                        "Something went wrong while trying to convert the value `{}` in tag {}."
                    ),
                    self.textual_value, self.xml_helper.tag
                )
            })?);
        }
        Ok(
            self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed,
        )
    }

    pub fn build(self) -> Option<T> {
        if self.textual_value.is_empty() {
            return None;
        }
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
        Ok(
            self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed,
        )
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
        Ok(
            self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed,
        )
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
            return Ok(self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed);
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
        Ok(
            self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed,
        )
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
            iso_abbreviation: self.iso_abbreviation_builder.build(),
            journal_issue: self.journal_issue_builder.build()?,
        })
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

#[derive(Debug)]
struct AbstractBuilder {
    xml_helper: XMLHelper,
    abstract_test: Vec<String>,
    abstract_builder: ObjectBuilder<String>,
}

impl AbstractBuilder {
    pub fn new(tag: &str) -> Self {
        AbstractBuilder {
            xml_helper: XMLHelper::new(tag),
            abstract_test: Vec::new(),
            abstract_builder: ObjectBuilder::new("AbstractText"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if line.is_empty() {
            return Ok(self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed);
        }
        self.abstract_builder.parse(line)?;
        if self.abstract_builder.can_build() {
            self.abstract_test.push(
                core::mem::replace(
                    &mut self.abstract_builder,
                    ObjectBuilder::new("AbstractText"),
                )
                .build()
                .unwrap(),
            );
        }

        Ok(!self.xml_helper.tag_closed)
    }

    pub fn build(self) -> Result<Abstract, String> {
        if !self.xml_helper.can_build() {
            return Err(format!(concat!(
                "Build method was called on AbstractBuilder ",
                "but the object is not yet ready to build."
            )));
        }
        Ok(Abstract {
            language: self
                .xml_helper
                .attributes
                .get("Language")
                .map(|val| val.clone()),
            abstract_type: self
                .xml_helper
                .attributes
                .get("Type")
                .map(|val| val.clone()),
            text: self.abstract_test.join(" "),
        })
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
        Ok(
            self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed,
        )
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
            return Ok(self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed);
        }

        self.chemical_builder.parse(line)?;
        if self.chemical_builder.can_build() {
            self.chemicals.push(
                core::mem::replace(&mut self.chemical_builder, ChemicalBuilder::new())
                    .build()
                    .unwrap(),
            );
        }

        Ok(
            self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed,
        )
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
        Ok(
            self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed,
        )
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
            return Ok(self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed);
        }
        self.mesh_builder.parse(line)?;
        if self.mesh_builder.can_build() {
            self.meshes.push(
                core::mem::replace(&mut self.mesh_builder, MeshBuilder::new())
                    .build()
                    .unwrap(),
            );
        }

        Ok(
            self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed,
        )
    }

    pub fn build(self) -> Result<Vec<Mesh>, String> {
        if !self.xml_helper.can_build() && !self.meshes.is_empty() {
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
    owner: String,
    keyword_builder: ObjectBuilder<String>,
}

impl KeywordListBuilder {
    pub fn new(owner: &str) -> Self {
        KeywordListBuilder {
            xml_helper: XMLHelper::new("KeywordList"),
            keywords: Vec::new(),
            owner: owner.to_string(),
            keyword_builder: ObjectBuilder::with_attributes(
                "Keyword",
                [("Owner".to_string(), owner.to_string())]
                    .into_iter()
                    .collect(),
            ),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if line.is_empty() {
            return Ok(self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed);
        }
        self.keyword_builder.parse(line)?;
        if self.keyword_builder.can_build() {
            let keyword_builder = core::mem::replace(
                &mut self.keyword_builder,
                ObjectBuilder::with_attributes(
                    "Keyword",
                    [("Owner".to_string(), self.owner.clone())]
                        .into_iter()
                        .collect(),
                ),
            );
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

        Ok(
            self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed,
        )
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
struct ArticleIdsBuilder {
    xml_helper: XMLHelper,
    article_ids: Vec<ArticleId>,
    article_id_builder: ObjectBuilder<String>,
}

impl ArticleIdsBuilder {
    pub fn new() -> Self {
        ArticleIdsBuilder {
            xml_helper: XMLHelper::new("ArticleIdList"),
            article_ids: Vec::new(),
            article_id_builder: ObjectBuilder::new("ArticleId"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if line.is_empty() {
            return Ok(self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed);
        }
        if self.article_id_builder.parse(line).is_err() {
            let _ = core::mem::replace(
                &mut self.article_id_builder,
                ObjectBuilder::new("ArticleId"),
            );
        }
        if self.article_id_builder.can_build() {
            let article_id_builder = core::mem::replace(
                &mut self.article_id_builder,
                ObjectBuilder::new("ArticleId"),
            );
            self.article_ids.push(ArticleId {
                id_type: article_id_builder
                    .xml_helper
                    .attributes
                    .get("IdType")
                    .unwrap()
                    .clone(),
                value: article_id_builder.build().unwrap(),
            })
        }

        Ok(
            self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed,
        )
    }

    pub fn build(self) -> Result<Vec<ArticleId>, String> {
        if !self.xml_helper.can_build() && !self.article_ids.is_empty() {
            return Err(format!(
                concat!(
                    "Build method was called on ArticleIdListBuilder ",
                    "but the object is not yet ready to build. ",
                    "The object currently looks like {:?}"
                ),
                self
            ));
        }
        Ok(self.article_ids)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

#[derive(Debug)]
struct OtherAbstractBuilder {
    other_abstracts: Vec<Abstract>,
    other_abstract_builder: AbstractBuilder,
}

impl OtherAbstractBuilder {
    pub fn new() -> Self {
        OtherAbstractBuilder {
            other_abstracts: Vec::new(),
            other_abstract_builder: AbstractBuilder::new("OtherAbstract"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        self.other_abstract_builder.parse(line)?;
        let parsed = self.other_abstract_builder.xml_helper.tag_opened;
        if self.other_abstract_builder.can_build() {
            self.other_abstracts.push(
                core::mem::replace(
                    &mut self.other_abstract_builder,
                    AbstractBuilder::new("OtherAbstract"),
                )
                .build()?,
            );
        }
        Ok(parsed)
    }

    pub fn build(self) -> Result<Vec<Abstract>, String> {
        if self.other_abstract_builder.xml_helper.tag_opened
            && !self.other_abstract_builder.xml_helper.tag_closed
            && !self.other_abstracts.is_empty()
        {
            return Err(format!(
                concat!(
                    "Build method was called on OtherAbstractBuilder ",
                    "but the object is not yet ready to build. ",
                    "The object currently looks like {:?}"
                ),
                self
            ));
        }
        Ok(self.other_abstracts)
    }
}

#[derive(Debug)]
struct GeneSymbolListBuilder {
    xml_helper: XMLHelper,
    gene_symbols: Vec<String>,
    gene_symbol_builder: ObjectBuilder<String>,
}

impl GeneSymbolListBuilder {
    pub fn new() -> Self {
        GeneSymbolListBuilder {
            xml_helper: XMLHelper::new("GeneSymbolList"),
            gene_symbols: Vec::new(),
            gene_symbol_builder: ObjectBuilder::new("GeneSymbol"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if line.is_empty() {
            return Ok(self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed);
        }
        self.gene_symbol_builder.parse(line)?;
        if self.gene_symbol_builder.can_build() {
            let gene_symbol_builder = core::mem::replace(
                &mut self.gene_symbol_builder,
                ObjectBuilder::new("GeneSymbol"),
            );
            self.gene_symbols.push(gene_symbol_builder.build().unwrap())
        }

        Ok(
            self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed,
        )
    }

    pub fn build(self) -> Result<Vec<String>, String> {
        if !self.xml_helper.can_build() && !self.gene_symbols.is_empty() {
            return Err(format!(
                concat!(
                    "Build method was called on GeneSymbolListBuilder ",
                    "but the object is not yet ready to build. ",
                    "The object currently looks like {:?}"
                ),
                self
            ));
        }
        Ok(self.gene_symbols)
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}

#[derive(Debug)]
struct SupplMeshListBuilder {
    xml_helper: XMLHelper,
    meshes: Vec<SupplMesh>,
    suppl_mesh_builder: ObjectBuilder<String>,
}

impl SupplMeshListBuilder {
    pub fn new() -> Self {
        SupplMeshListBuilder {
            xml_helper: XMLHelper::new("SupplMeshList"),
            meshes: Vec::new(),
            suppl_mesh_builder: ObjectBuilder::new("SupplMeshName"),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let line = self.xml_helper.parse(line)?;
        if line.is_empty() {
            return Ok(self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed);
        }
        self.suppl_mesh_builder.parse(line)?;
        if self.suppl_mesh_builder.can_build() {
            let suppl_mesh_builder = core::mem::replace(
                &mut self.suppl_mesh_builder,
                ObjectBuilder::new("SupplMeshName"),
            );
            self.meshes.push(SupplMesh {
                code: suppl_mesh_builder
                    .xml_helper
                    .attributes
                    .get("UI")
                    .unwrap()
                    .clone(),
                mesh_type: suppl_mesh_builder
                    .xml_helper
                    .attributes
                    .get("Type")
                    .unwrap()
                    .clone(),
                name: suppl_mesh_builder.build().unwrap(),
            })
        }

        Ok(
            self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed,
        )
    }

    pub fn build(self) -> Result<Vec<SupplMesh>, String> {
        if !self.xml_helper.can_build() && !self.meshes.is_empty() {
            return Err(format!(
                concat!(
                    "Build method was called on SupplMeshListBuilder ",
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
struct ReferencesBuilder {
    xml_helper: XMLHelper,
    references: Vec<usize>,
    pubmed_builder: ObjectBuilder<usize>,
}

impl ReferencesBuilder {
    pub fn new() -> Self {
        ReferencesBuilder {
            xml_helper: XMLHelper::with_reopening("ReferenceList"),
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
            return Ok(self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed);
        }
        if self.pubmed_builder.parse(line).is_err() {
            let _ = core::mem::replace(
                &mut self.pubmed_builder,
                ObjectBuilder::with_attributes(
                    "ArticleId",
                    [("IdType".to_string(), "pubmed".to_string())]
                        .into_iter()
                        .collect(),
                )
            );
        }
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
        Ok(
            self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed,
        )
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
}

struct IgnoreTag {
    xml_helper: XMLHelper,
}

impl IgnoreTag {
    pub fn new(tag: &str) -> Self {
        IgnoreTag {
            xml_helper: XMLHelper::new(tag),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        let _ = self.xml_helper.parse(line)?;
        Ok(
            self.xml_helper.tag_opened && !self.xml_helper.tag_closed
                || self.xml_helper.just_closed,
        )
    }
}

struct IgnoreTags {
    ignored_tags: Vec<IgnoreTag>,
}

impl IgnoreTags {
    pub fn new(tags: &[&str]) -> Self {
        IgnoreTags {
            ignored_tags: tags.iter().map(|tag| IgnoreTag::new(tag)).collect(),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<bool, String> {
        for ignored_tag in self.ignored_tags.iter_mut() {
            if ignored_tag.parse(line)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

pub(crate) struct ArticleBuilder {
    xml_helper: XMLHelper,
    completion_date_builder: DateBuilder,
    revised_date_builder: DateBuilder,
    pmid_builder: ObjectBuilder<u32>,
    article_ids_builder: ArticleIdsBuilder,
    journal_builder: JournalBuilder,
    title_builder: ObjectBuilder<String>,
    abstract_text_builder: AbstractBuilder,
    other_abstracts_builders: OtherAbstractBuilder,
    language_builder: ObjectBuilder<String>,
    chemical_list_builder: ChemicalListBuilder,
    mesh_list_builder: MeshListBuilder,
    suppl_mesh_list_builder: SupplMeshListBuilder,
    references_builder: ReferencesBuilder,
    pip_keywords_builder: KeywordListBuilder,
    kie_keywords_builder: KeywordListBuilder,
    gene_symbol_list_builder: GeneSymbolListBuilder,
    ignored_tags: IgnoreTags,
}

impl ArticleBuilder {
    pub fn new() -> Self {
        ArticleBuilder {
            xml_helper: XMLHelper::new("PubmedArticle"),
            completion_date_builder: DateBuilder::new("DateCompleted"),
            revised_date_builder: DateBuilder::new("DateRevised"),
            pmid_builder: ObjectBuilder::new("PMID"),
            article_ids_builder: ArticleIdsBuilder::new(),
            journal_builder: JournalBuilder::new(),
            title_builder: ObjectBuilder::new("ArticleTitle"),
            abstract_text_builder: AbstractBuilder::new("Abstract"),
            other_abstracts_builders: OtherAbstractBuilder::new(),
            language_builder: ObjectBuilder::new("Language"),
            chemical_list_builder: ChemicalListBuilder::new(),
            mesh_list_builder: MeshListBuilder::new(),
            suppl_mesh_list_builder: SupplMeshListBuilder::new(),
            references_builder: ReferencesBuilder::new(),
            pip_keywords_builder: KeywordListBuilder::new("PIP"),
            kie_keywords_builder: KeywordListBuilder::new("KIE"),
            gene_symbol_list_builder: GeneSymbolListBuilder::new(),
            ignored_tags: IgnoreTags::new(&[
                "AuthorList",
                "PublicationTypeList",
                "MedlineJournalInfo",
                "History",
                "PersonalNameSubjectList",
                "DataBankList",
                "GrantList",
                "CoiStatement",
                "VernacularTitle",
                "CommentsCorrectionsList",
                "ArticleDate",
                "InvestigatorList",
            ]),
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<(), String> {
        //println!("{}", line);
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
        if !self.article_ids_builder.can_build() && self.article_ids_builder.parse(line)? {
            return Ok(());
        }
        if !self.journal_builder.can_build() && self.journal_builder.parse(line)? {
            return Ok(());
        }
        if !self.title_builder.can_build() && self.title_builder.parse(line)? {
            return Ok(());
        }
        if !self.abstract_text_builder.can_build() && self.abstract_text_builder.parse(line)? {
            return Ok(());
        }
        if self.other_abstracts_builders.parse(line)? {
            return Ok(());
        }
        if self.ignored_tags.parse(line)? {
            return Ok(());
        }
        if self.language_builder.parse(line)? {
            return Ok(());
        }
        if !self.chemical_list_builder.can_build() && self.chemical_list_builder.parse(line)? {
            return Ok(());
        }
        if !self.gene_symbol_list_builder.can_build()
            && self.gene_symbol_list_builder.parse(line)?
        {
            return Ok(());
        }
        if !self.mesh_list_builder.can_build() && self.mesh_list_builder.parse(line)? {
            return Ok(());
        }
        if !self.suppl_mesh_list_builder.can_build() && self.suppl_mesh_list_builder.parse(line)? {
            return Ok(());
        }
        if !self.pip_keywords_builder.can_build() && self.pip_keywords_builder.parse(line)? {
            return Ok(());
        }
        if !self.kie_keywords_builder.can_build() && self.kie_keywords_builder.parse(line)? {
            return Ok(());
        }
        if self.references_builder.parse(line)? {
            return Ok(());
        }

        Err(format!("The line {} was not handled by any parser.", line))
    }

    pub fn build(self) -> Result<Article, String> {
        if !self.xml_helper.can_build() {
            return Err("The article is not ready!".to_string());
        }
        let mut keywords = self.pip_keywords_builder.build()?;

        keywords.extend(self.kie_keywords_builder.build()?);
        Ok(Article {
            completion_date: self.completion_date_builder.build().ok(),
            revision_date: self.revised_date_builder.build().ok(),
            pubmed_id: self.pmid_builder.build().unwrap(),
            article_ids: self.article_ids_builder.build()?,
            journal: self.journal_builder.build()?,
            title: self.title_builder.build(),
            abstract_text: self.abstract_text_builder.build().ok(),
            other_abstract_texts: self.other_abstracts_builders.build()?,
            chemical_list: self.chemical_list_builder.build()?,
            mesh_list: self.mesh_list_builder.build()?,
            gene_symbol_list: self.gene_symbol_list_builder.build()?,
            suppl_mesh_list: self.suppl_mesh_list_builder.build()?,
            references: self.references_builder.build()?,
            keywords,
        })
    }

    pub fn can_build(&self) -> bool {
        self.xml_helper.can_build()
    }
}
