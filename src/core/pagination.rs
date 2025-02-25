use itertools::Itertools;
use tracing::trace;

#[cfg_attr(feature = "dtos", derive(serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Page {
    page: usize,
    size: usize,
}

impl Page {
    pub fn new(page: usize, size: usize) -> Self {
        assert!(page > 0);
        assert!(size >= 1);
        Self { page, size }
    }
    pub fn number(&self) -> usize {
        self.page
    }
    pub fn offset(&self) -> usize {
        if self.page <= 1 {
            0
        } else {
            (self.page - 1) * self.size
        }
    }
    pub fn page_size(&self) -> usize {
        self.size
    }
}

impl Default for Page {
    fn default() -> Self {
        Self { page: 1, size: 10 }
    }
}

/// Paged data response
#[derive(Debug, Clone)]
#[cfg_attr(feature = "dtos", derive(serde::Serialize))]
pub struct Paged<T> {
    pub page: usize,
    pub data: Vec<T>,
}
impl<T> Default for Paged<T> {
    fn default() -> Self {
        Paged {
            page: 1,
            data: vec![],
        }
    }
}
impl<T> Paged<T> {
    fn _len(&self) -> usize {
        self.data.len()
    }
}
impl<T> std::fmt::Display for Paged<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "---\n * Page {:02} *\n{}\n---",
            self.page,
            self.data.iter().join("\n")
        )
    }
}

/// A trait for model providing pages
pub trait Paginable {
    type Output;
    /// Returns the **subset** of data corresponding to the page requested
    fn get_page(&self, page: &Page) -> Paged<Self::Output>;
    /// Returns list with the page info (does not limit shorten the list)
    fn to_paged(self, page: Page) -> Paged<Self::Output>;
}

// defualt vec impl
impl<T> Paginable for Vec<T>
where
    T: Clone,
{
    type Output = T;
    fn get_page(&self, page: &Page) -> Paged<Self::Output> {
        let Page { page, size } = page;
        if self.len() < *size {
            return Paged {
                page: 1,
                data: self.to_vec(),
            };
        }
        let pages_count = (self.len() / size) + 1;
        let page = if *page > pages_count {
            pages_count
        } else {
            *page
        };
        let offset = (page - 1) * size;
        Paged {
            page,
            data: self[offset..(offset + size)].to_vec(),
        }
    }
    fn to_paged(self, page: Page) -> Paged<Self::Output> {
        assert!(page.page_size() >= 1, "required page size is 0. WTF.");
        assert!(
            self.len() <= page.page_size(),
            "There are more elements than required by page size: ps:{}, len:{}",
            page.page_size(),
            self.len()
        );
        trace!("Returning page from list of {} elements", self.len());
        Paged {
            page: page.number(),
            data: self,
        }
    }
}
