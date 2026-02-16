use std::fs::File;
use std::io::{self, Write, Seek, BufWriter};

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

pub trait WriteSeek: Write + Seek {}
impl<T: Write + Seek> WriteSeek for T {}

pub struct PdfWriter<W: WriteSeek = Box<dyn WriteSeek>> {
    writer: BufWriter<W>,
    offset: u64,
    pub(crate) xref: Vec<(u32, u64)>, // id -> offset
}

impl<W: WriteSeek> PdfWriter<W> {
    pub fn new(writer: W) -> io::Result<Self> {
        let mut writer = BufWriter::with_capacity(64 * 1024, writer); // 64KB buffer
        
        // Write Header
        let header = b"%PDF-1.7\n%\x93\x8C\x8B\x9E\n"; // Binary comment to indicate binary file
        writer.write_all(header)?;
        
        Ok(PdfWriter {
            writer,
            offset: header.len() as u64,
            xref: Vec::new(),
        })
    }

}

impl PdfWriter<Box<dyn WriteSeek>> {
    pub fn from_path(path: &str) -> io::Result<Self> {
        let file = File::create(path)?;
        Self::new(Box::new(file))
    }
}

impl<W: WriteSeek> PdfWriter<W> {


    pub fn write_object(&mut self, id: u32, object: &PdfObject) -> io::Result<()> {
       self.xref.push((id, self.offset));
       
       let start_offset = self.offset;
       
       // Write object to buffer
       write!(self.writer, "{} 0 obj\n", id)?;
       object.serialize(&mut self.writer)?;
       write!(self.writer, "\nendobj\n")?;
       
       // Calculate offset increment without flushing
       // This is an approximation but works for tracking
       let obj_header = format!("{} 0 obj\n", id);
       let obj_footer = "\nendobj\n";
       
       // Estimate size (not perfect but close enough for xref)
       // We'll flush and get exact position only when needed
       self.offset += (obj_header.len() + obj_footer.len()) as u64;
       
       // For now, we need exact positions, so flush
       // TODO: Optimize by batching writes
       self.writer.flush()?;
       self.offset = self.writer.stream_position()?;
       
       Ok(())
    }





    pub fn write_xref_and_trailer(&mut self, root_id: u32) -> io::Result<()> {
        let xref_offset = self.offset;
        
        // Sort XREF by ID to ensure the table corresponds to the implicit object numbering (1, 2, 3...)
        // This is critical for streaming mode where objects are written out of order (e.g. Pages object #2 is written last)
        self.xref.sort_by_key(|&(id, _)| id);
        
        // Xref
        writeln!(self.writer, "xref")?;
        writeln!(self.writer, "0 {}", self.xref.len() + 1)?; // +1 for the 0th object
        
        // Entry 0
        writeln!(self.writer, "0000000000 65535 f ")?;
        
        for (_id, offset) in &self.xref {
            writeln!(self.writer, "{:010} 00000 n ", offset)?;
        }
        
        // Trailer
        writeln!(self.writer, "trailer")?;
        write!(self.writer, "<< /Size {} /Root {} 0 R >>", self.xref.len() + 1, root_id)?;
        
        writeln!(self.writer, "\nstartxref")?;
        writeln!(self.writer, "{}", xref_offset)?;
        writeln!(self.writer, "%%EOF")?;
        
        // Final flush
        self.writer.flush()?;
        
        Ok(())
    }
}
