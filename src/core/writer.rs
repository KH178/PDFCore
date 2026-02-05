use std::fs::File;
use std::io::{self, Write, Seek};

/// Core PDF Objects based on PDF Reference 1.7
#[derive(Debug, Clone)]
pub enum PdfObject {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    Name(String),
    String(String),
    Array(Vec<PdfObject>),
    Dictionary(Vec<(String, PdfObject)>),
    Stream(Vec<(String, PdfObject)>, Vec<u8>), // Dictionary + Content
    Reference(u32), // Indirect Object Reference (id)
}

impl PdfObject {
    /// Serializes the object to the writer
    pub fn serialize<W: Write>(&self, w: &mut W) -> io::Result<()> {
        match self {
            PdfObject::Null => write!(w, "null"),
            PdfObject::Boolean(b) => write!(w, "{}", b),
            PdfObject::Integer(i) => write!(w, "{}", i),
            PdfObject::Real(r) => write!(w, "{}", r),
            PdfObject::Name(n) => write!(w, "/{}", n),
            PdfObject::String(s) => write!(w, "({})", escape_string(s)), // Basic escaping needed
            PdfObject::Array(arr) => {
                write!(w, "[")?;
                for (i, obj) in arr.iter().enumerate() {
                    if i > 0 { write!(w, " ")?; }
                    obj.serialize(w)?;
                }
                write!(w, "]")
            }
            PdfObject::Dictionary(dict) => {
                write!(w, "<<")?;
                for (key, val) in dict {
                    write!(w, " /{} ", key)?;
                    val.serialize(w)?;
                }
                write!(w, " >>")
            }
            PdfObject::Stream(dict, content) => {
                write!(w, "<<")?;
                for (key, val) in dict {
                    write!(w, " /{} ", key)?;
                    val.serialize(w)?;
                }
                // Ensure Length is correct for the stream
                write!(w, " /Length {} >>\nstream\n", content.len())?;
                w.write_all(content)?;
                write!(w, "\nendstream")
            }
            PdfObject::Reference(id) => write!(w, "{} 0 R", id),
        }
    }
}

pub fn escape_string(s: &str) -> String {
    s.replace("\\", "\\\\").replace("(", "\\(").replace(")", "\\)")
}

pub struct PdfWriter {
    file: File,
    offset: u64,
    pub(crate) xref: Vec<(u32, u64)>, // id -> offset
}

impl PdfWriter {
    pub fn new(path: &str) -> io::Result<Self> {
        let mut file = File::create(path)?;
        // Write Header
        let header = b"%PDF-1.7\n%\x93\x8C\x8B\x9E\n"; // Binary comment to indicate binary file
        file.write_all(header)?;
        
        Ok(PdfWriter {
            file,
            offset: header.len() as u64,
            xref: Vec::new(),
        })
    }

    pub fn write_object(&mut self, id: u32, object: &PdfObject) -> io::Result<()> {
       self.xref.push((id, self.offset));
       
       write!(self.file, "{} 0 obj\n", id)?;
       object.serialize(&mut self.file)?;
       write!(self.file, "\nendobj\n")?;
       
       self.offset = self.file.stream_position()?;
       Ok(())
    }


    pub fn write_xref_and_trailer(&mut self, root_id: u32) -> io::Result<()> {
        let xref_offset = self.offset;
        
        // Sort XREF by ID to ensure the table corresponds to the implicit object numbering (1, 2, 3...)
        // This is critical for streaming mode where objects are written out of order (e.g. Pages object #2 is written last)
        self.xref.sort_by_key(|&(id, _)| id);
        
        // Xref
        writeln!(self.file, "xref")?;
        writeln!(self.file, "0 {}", self.xref.len() + 1)?; // +1 for the 0th object
        
        // Entry 0
        writeln!(self.file, "0000000000 65535 f ")?;
        
        for (_id, offset) in &self.xref {
            writeln!(self.file, "{:010} 00000 n ", offset)?;
        }
        
        // Trailer
        writeln!(self.file, "trailer")?;
        write!(self.file, "<< /Size {} /Root {} 0 R >>", self.xref.len() + 1, root_id)?;
        
        writeln!(self.file, "\nstartxref")?;
        writeln!(self.file, "{}", xref_offset)?;
        writeln!(self.file, "%%EOF")?;
        
        Ok(())
    }
}
