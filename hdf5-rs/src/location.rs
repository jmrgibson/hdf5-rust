use std::ptr;

use libhdf5_sys::{
    h5f::H5Fget_name,
    h5i::{H5Iget_file_id, H5Iget_name},
    h5o::{H5Oget_comment, H5Oset_comment},
};

use crate::internal_prelude::*;

/// Named location (file, group, dataset, named datatype).
def_object_class!(
    Location: Object,
    "location",
    &[H5I_FILE, H5I_GROUP, H5I_DATATYPE, H5I_DATASET, H5I_ATTR] as &[_],
    &Location::repr
);

impl Location {
    /// Returns the name of the object within the file, or empty string if the object doesn't
    /// have a name (e.g., an anonymous dataset).
    pub fn name(&self) -> String {
        // TODO: should this return Result<String> or an empty string if it fails?
        h5lock!(get_h5_str(|m, s| H5Iget_name(self.id(), m, s)).unwrap_or_else(|_| "".to_string()))
    }

    /// Returns the name of the file containing the named object (or the file itself).
    pub fn filename(&self) -> String {
        // TODO: should this return Result<String> or an empty string if it fails?
        h5lock!(get_h5_str(|m, s| H5Fget_name(self.id(), m, s)).unwrap_or_else(|_| "".to_string()))
    }

    /// Returns a handle to the file containing the named object (or the file itself).
    pub fn file(&self) -> Result<File> {
        File::from_id(h5try!(H5Iget_file_id(self.id())))
    }

    /// Returns the commment attached to the named object, if any.
    pub fn comment(&self) -> Option<String> {
        // TODO: should this return Result<Option<String>> or fail silently?
        let comment = h5lock!(get_h5_str(|m, s| H5Oget_comment(self.id(), m, s)).ok());
        comment.and_then(|c| if c.is_empty() { None } else { Some(c) })
    }

    /// Set or the commment attached to the named object.
    pub fn set_comment(&self, comment: &str) -> Result<()> {
        // TODO: &mut self?
        let comment = to_cstring(comment)?;
        h5call!(H5Oset_comment(self.id(), comment.as_ptr())).and(Ok(()))
    }

    /// Clear the commment attached to the named object.
    pub fn clear_comment(&self) -> Result<()> {
        // TODO: &mut self?
        h5call!(H5Oset_comment(self.id(), ptr::null_mut())).and(Ok(()))
    }

    fn repr(&self) -> String {
        format!("\"{}\"", self.name())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::internal_prelude::*;

    #[test]
    pub fn test_filename() {
        with_tmp_path(|path| {
            assert_eq!(File::open(&path, "w").unwrap().filename(), path.to_str().unwrap());
        })
    }

    #[test]
    pub fn test_name() {
        with_tmp_file(|file| {
            assert_eq!(file.name(), "/");
        })
    }

    #[test]
    pub fn test_file() {
        with_tmp_file(|file| {
            assert_eq!(file.file().unwrap().id(), file.id());
        })
    }

    #[test]
    pub fn test_comment() {
        with_tmp_file(|file| {
            assert!(file.comment().is_none());
            assert!(file.set_comment("foo").is_ok());
            assert_eq!(file.comment().unwrap(), "foo");
            assert!(file.clear_comment().is_ok());
            assert!(file.comment().is_none());
        })
    }
}
