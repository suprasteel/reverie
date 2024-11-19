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
        if self.page == 0 {
            0
        } else {
            (self.page - 1) * self.size
        }
    }
    pub fn page_size(&self) -> usize {
        self.page
    }
}

impl Default for Page {
    fn default() -> Self {
        Self { page: 1, size: 10 }
    }
}

/// Paged data response
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
        assert!(page.page_size() >= self.len());
        Paged {
            page: page.number(),
            data: self,
        }
    }
}
