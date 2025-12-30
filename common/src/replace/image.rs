use flate2::Crc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum TextWrapType {
    Embed,     //嵌入型
    BelowText, //嵌于文字下方
    AboveText, //嵌于文字上方
}

pub struct Extent {
    pub cy: f32,
    pub cx: f32,
}

pub struct Image {
    pub file: Box<[u8]>,
    pub relationship: &'static str,
    pub id: String,
    pub suffix: String,

    wp_extent: Extent,       //图片宽高
    text_wrap: TextWrapType, //文字环绕
}

const RELATIONSHIP: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";

pub fn generate_id(data: &Vec<u8>) -> String {
    let len = data.len();
    let mut crc = Crc::new();
    crc.update(data);
    format!("rId{}_{}", crc.sum().to_string(), &*len.to_string())
}

pub fn new(
    id: String,
    file: Box<[u8]>,
    suffix: String,
    text_wrap: TextWrapType,
    wp_extent: Extent,
) -> Image {
    Image {
        file,
        relationship: RELATIONSHIP,
        id,
        suffix,
        wp_extent,
        text_wrap,
    }
}

impl Image {
    pub fn clone(&self) -> Image {
        Image {
            file: self.file.clone(),
            relationship: self.relationship,
            id: self.id.clone(),
            suffix: self.suffix.clone(),
            wp_extent: Extent {
                cx: self.wp_extent.cx,
                cy: self.wp_extent.cy,
            },
            text_wrap: self.text_wrap.clone(),
        }
    }

    pub fn to_string(&self) -> String {
        let mut str_vec = vec![
            r#"</w:t></w:r><w:r><w:drawing>"#
        ];

        let tags = format!(
            r#"<wp:extent cx="{}" cy="{}"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:blipFill><a:blip r:embed="{}"/></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0" /><a:ext cx="{}" cy="{}" /></a:xfrm><a:prstGeom prst="rect"><a:avLst /></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic>"#,
            self.wp_extent.cx, self.wp_extent.cy, self.id, self.wp_extent.cx, self.wp_extent.cy
        );

        match &self.text_wrap {
            TextWrapType::Embed => {
                str_vec.push(r#"<wp:inline distT="0" distB="0" distL="0" distR="0">"#);
                str_vec.push(&tags);
                str_vec.push(r#"</wp:inline>"#);
            }
            TextWrapType::BelowText => {
                str_vec.push(r#"<wp:anchor distT="0" distB="0" distL="0" distR="0" simplePos="0" behindDoc="1" locked="0" layoutInCell="1" allowOverlap="1"><wp:positionH relativeFrom="character"><wp:posOffset>0</wp:posOffset></wp:positionH><wp:positionV relativeFrom="line"><wp:posOffset>0</wp:posOffset></wp:positionV>"#);
                str_vec.push(&tags);
                str_vec.push(r#"</wp:anchor>"#);
            }
            TextWrapType::AboveText => {
                str_vec.push(r#"<wp:anchor distT="0" distB="0" distL="0" distR="0" simplePos="0" behindDoc="0" locked="0" layoutInCell="1" allowOverlap="1"><wp:positionH relativeFrom="character"><wp:posOffset>0</wp:posOffset></wp:positionH><wp:positionV relativeFrom="line"><wp:posOffset>0</wp:posOffset></wp:positionV>"#);
                str_vec.push(&tags);
                str_vec.push(r#"</wp:anchor>"#);
            }
        }
        str_vec.push(r#"</w:drawing></w:r><w:r><w:t>"#);
        str_vec.join("")
    }
}
