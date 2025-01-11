use crate::office::office::Office;
use crate::replace::image::generate_id;
use async_zip::base::read::mem::ZipFileReader;
use async_zip::error::ZipError;
use futures::future::join_all;
use std::collections::HashMap;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

pub struct RelationshipInfo {
    pub id: String,
    pub target: String,
    pub _type: String,
}

pub struct Zip {
    reader: ZipFileReader,
    files: HashMap<String, Box<[u8]>>,
    archive: ZipArchive<Cursor<Vec<u8>>>,
    office: Box<dyn Office>,
    relationships: HashMap<String, Vec<RelationshipInfo>>,
}

pub async fn new(file: Vec<u8>, office: Box<dyn Office>) -> Result<Zip, ZipError> {
    Ok(Zip {
        office,
        reader: ZipFileReader::new(file.clone()).await?,
        files: HashMap::new(),
        archive: ZipArchive::new(Cursor::new(file)).unwrap(),
        relationships: HashMap::new(),
    })
}

impl Zip {
    pub async fn get_document_content(&self, index: usize) -> Option<Box<[u8]>> {
        if let Ok(mut reader) = self.reader.reader_with_entry(index).await {
            let mut bytes = Vec::new();
            if let Ok(_) = reader.read_to_end_checked(&mut bytes).await {
                return Some(bytes.into_boxed_slice());
            }
        }
        None
    }

    pub async fn get_document_content_by_name(&self, file_name: &str) -> Option<Box<[u8]>> {
        let entries = self.reader.file().entries();
        for (i, entry) in entries.iter().enumerate() {
            let entry_file_name = entry.filename().as_str().unwrap();
            if entry_file_name.ends_with('/') {
                continue;
            }
            if !file_name.eq(entry_file_name) {
                continue;
            }
            return self.get_document_content(i).await;
        }
        None
    }

    pub async fn match_document_names(&self) -> Vec<&str> {
        let entries = self.reader.file().entries();
        let mut file_names = Vec::new();
        let regex = self.office.regex();
        for entry in entries {
            let file_name = entry.filename().as_str().unwrap();
            if file_name.ends_with('/') {
                continue;
            }
            if regex.is_match(file_name) {
                file_names.push(file_name);
            }
        }
        file_names
    }

    pub async fn match_document_contents(&self) -> Option<HashMap<String, Box<[u8]>>> {
        let entries = self.reader.file().entries();
        let mut tasks = Vec::new();
        let mut names = Vec::new();
        let regex = self.office.regex();
        for (i, entry) in entries.iter().enumerate() {
            let file_name = entry.filename().as_str().unwrap();
            if file_name.ends_with('/') {
                continue;
            }
            if regex.is_match(file_name) {
                names.push(file_name);
                tasks.push(self.get_document_content(i));
            }
        }
        if tasks.is_empty() {
            return None;
        }
        let mut res = join_all(tasks).await;
        let mut files: HashMap<String, Box<[u8]>> = HashMap::new();
        while let Some(file) = res.pop() {
            let name = names.pop().unwrap();
            if let Some(file) = file {
                files.insert(name.parse().unwrap(), file);
            }
        }
        Some(files)
    }

    pub fn write_file(&mut self, file_name: String, file_data: Box<[u8]>) {
        self.files.insert(file_name, file_data);
    }

    async fn get_relationship(&self, file_name: &String) -> Box<[u8]> {
        if let Some(content) = self.get_document_content_by_name(file_name).await {
            content
        } else {
            Box::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"></Relationships>"#.as_bytes())
        }
    }

    pub fn write_relationships(&mut self, file_name: String, relationships: Vec<RelationshipInfo>) {
        self.relationships.insert(file_name, relationships);
    }

    pub async fn get_medias(&self) -> Option<HashMap<String, (String, Box<[u8]>)>> {
        let entries = self.reader.file().entries();
        let media_dir = self.office.root_dir().to_owned() + "media/";

        let mut tasks = Vec::new();
        let mut names = vec![];
        for (i, entry) in entries.iter().enumerate() {
            let file_name = entry.filename().as_str().unwrap();
            if !file_name.starts_with(&media_dir) || file_name.ends_with('/') {
                continue;
            }
            names.push(file_name);
            tasks.push(self.get_document_content(i));
        }

        let mut res = join_all(tasks).await;
        let mut media_map = HashMap::new();
        while let Some(file) = res.pop() {
            let name = names.pop().unwrap().to_string();
            if let Some(file) = file {
                let id = generate_id(&file.to_vec());
                media_map.insert(id, (name, file));
            }
        }
        if media_map.is_empty() {
            return None;
        }
        Some(media_map)
    }

    pub async fn get_media_names(&self) -> Option<HashMap<String, String>> {
        let entries = self.reader.file().entries();
        let media_dir = self.office.root_dir().to_owned() + "media/";

        let mut tasks = Vec::new();
        let mut names = vec![];
        for (i, entry) in entries.iter().enumerate() {
            let file_name = entry.filename().as_str().unwrap();
            if !file_name.starts_with(&media_dir) || file_name.ends_with('/') {
                continue;
            }
            names.push(file_name);
            tasks.push(self.get_document_content(i));
        }

        let mut res = join_all(tasks).await;
        let mut media_map = HashMap::new();
        while let Some(file) = res.pop() {
            let name = names.pop().unwrap().to_string();
            if let Some(file) = file {
                let id = generate_id(&file.to_vec());
                media_map.insert(id, name);
            }
        }
        if media_map.is_empty() {
            return None;
        }
        Some(media_map)
    }

    pub fn write_media(&mut self, file_name: String, file_data: Box<[u8]>) {
        self.files
            .insert(self.office.root_dir().to_owned() + "media/" + &*file_name, file_data);
    }

    pub async fn finish(&mut self) -> Box<[u8]> {
        let mut buffer = Vec::new();
        let mut writer = ZipWriter::new(Cursor::new(&mut buffer));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for (name, relationships) in self.relationships.iter() {
            if relationships.is_empty() {
                continue;
            }

            let name = self.office.root_dir().to_owned() + "_rels/" + name;
            let content = self.get_relationship(&name).await;
            if let Ok(mut str) = String::from_utf8(content.to_vec()) {
                let mut relationships_text = String::new();
                for relationship in relationships {
                    relationships_text.push_str(&format!(r#"<Relationship Id="{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/{}"/>"#, relationship.id, relationship.target))
                }
                match str.rfind("</Relationships>") {
                    Some(index) => {
                        str.insert_str(index, &relationships_text);
                    }
                    None => {
                        str.push_str(&relationships_text);
                    }
                }
                self.files.insert(name, str.into_bytes().into_boxed_slice());
            }
        }

        for (name, file) in self.files.iter() {
            writer.start_file(name, options).unwrap();
            writer.write_all(file).unwrap();
        }

        for i in 0..self.archive.len() {
            let entry = self.archive.by_index(i).unwrap();
            let name = entry.name().to_string();

            if !self.files.contains_key(&name) {
                writer.raw_copy_file(entry).unwrap();
            }
        }

        writer.finish().unwrap();
        buffer.into_boxed_slice()
    }
}
