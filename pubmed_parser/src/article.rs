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
    pub(crate) language: Option<String>,
}
#[derive(Debug)]
pub struct ArticleId {
    pub(crate) id_type: String,
    pub(crate) value: String,
}

pub struct Node {
    pub(crate) node_name: String,
    pub(crate) node_type: String,
    pub(crate) description: String,
}

pub struct Edge {
    pub(crate) subject: String,
    pub(crate) object: String,
    pub(crate) edge_type: String,
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

impl Article {
    pub fn to_nodes(&self) -> Vec<Node> {
        let mut nodes = vec![Node {
            node_name: self.pubmed_id.to_string(),
            node_type: "Paper".to_string(),
            description: format!(
                "{}|{}|{}",
                self.title.as_ref().unwrap_or(&"|".to_string()),
                self.abstract_text
                    .as_ref()
                    .map(|abs| abs.text.to_string())
                    .unwrap_or("".to_string()),
                self.other_abstract_texts
                    .iter()
                    .map(|abs| { abs.text.to_string() })
                    .collect::<Vec<String>>()
                    .join("|")
            )
            .trim_matches('|')
            .to_string(),
        }];

        for chemical in self.chemical_list.iter() {
            nodes.push(Node {
                node_name: chemical.code.clone(),
                node_type: "Chemical".to_string(),
                description: chemical.name_of_substance.clone(),
            })
        }

        for gene in self.gene_symbol_list.iter() {
            nodes.push(Node {
                node_name: gene.clone(),
                node_type: "Gene".to_string(),
                description: "".to_string(),
            })
        }

        for mesh in self.mesh_list.iter() {
            nodes.push(Node {
                node_name: mesh.descriptor.code.clone(),
                node_type: "Mesh".to_string(),
                description: mesh.descriptor.name.clone(),
            });
            if let Some(qualifier) = &mesh.qualifier {
                nodes.push(Node {
                    node_name: qualifier.code.clone(),
                    node_type: "Mesh".to_string(),
                    description: qualifier.name.clone(),
                });
            }
        }

        for suppl_mesh in self.suppl_mesh_list.iter() {
            nodes.push(Node {
                node_name: suppl_mesh.code.clone(),
                node_type: suppl_mesh.mesh_type.clone(),
                description: suppl_mesh.name.clone(),
            });
        }

        for keyword in self.keywords.iter() {
            nodes.push(Node {
                node_name: keyword.name.clone(),
                node_type: "Keyword".to_string(),
                description: "".to_string(),
            });
        }

        nodes
    }

    pub fn to_edges(&self) -> Vec<Edge> {
        let mut edges = vec![];

        for chemical in self.chemical_list.iter() {
            edges.push(Edge {
                subject: self.pubmed_id.to_string(),
                object: chemical.code.clone(),
                edge_type: "PaperToChemical".to_string(),
            });
        }

        for gene in self.gene_symbol_list.iter() {
            edges.push(Edge {
                subject: self.pubmed_id.to_string(),
                object: gene.clone(),
                edge_type: "PaperToGene".to_string(),
            });
        }

        for mesh in self.mesh_list.iter() {
            edges.push(Edge {
                subject: self.pubmed_id.to_string(),
                object: mesh.descriptor.code.clone(),
                edge_type: "PaperToMesh".to_string(),
            });
            if let Some(qualifier) = &mesh.qualifier {
                edges.push(Edge {
                    subject: self.pubmed_id.to_string(),
                    object: qualifier.code.clone(),
                    edge_type: "PaperToMesh".to_string(),
                });
            }
        }

        for suppl_mesh in self.suppl_mesh_list.iter() {
            edges.push(Edge {
                subject: self.pubmed_id.to_string(),
                object: suppl_mesh.code.clone(),
                edge_type: format!("PaperTo{}", suppl_mesh.mesh_type),
            });
        }

        for keyword in self.keywords.iter() {
            edges.push(Edge {
                subject: self.pubmed_id.to_string(),
                object: keyword.name.clone(),
                edge_type: "PaperToKeyword".to_string(),
            });
        }

        for reference in self.references.iter() {
            edges.push(Edge {
                subject: self.pubmed_id.to_string(),
                object: reference.to_string(),
                edge_type: "Citation".to_string(),
            });
        }

        edges
    }
}
