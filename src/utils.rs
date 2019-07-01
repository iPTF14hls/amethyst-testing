use std::{
    ops::{Index, IndexMut},
    iter::IntoIterator,
};

pub struct Array2d<T: Default + Clone> {
    data: Vec<T>,
    dim: (usize, usize),
}

impl<T: Default + Clone + Copy> Array2d<T> {
    pub fn new(dim: (usize, usize)) -> Array2d<T> {
        let len = dim.0 * dim.1;

        Array2d{
            data: vec![T::default(); len],
            dim
        }
    }

    //These functions pull double duty.
    //They not only do a conversion but also bounds checking.
    pub fn pos_to_i(&self, (x, y):(usize, usize)) -> Option<usize> {
        let (wid, hei) = self.dim;
        if x < wid && y < hei {
            Some(wid*y+x)
        }
        else {
            None
        }
    }

    pub fn i_to_pos(&self, i: usize) -> Option<(usize, usize)> {
        let (wid, hei) = self.dim;
        let total = wid * hei;
        if i < total {
            Some((i%wid, i/wid))
        }
        else {
            None
        }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        self.dim
    }

    pub fn try_index(&self, pos: (usize, usize)) -> Option<T> {
        self.pos_to_i(pos).map(|i|self.data[i])
    }
}

impl<T: Default + Clone + Copy> Index<(usize, usize)> for Array2d<T> {
    type Output = T;

    fn index(&self, pos: (usize, usize)) -> &T {
        let i = self.pos_to_i(pos).expect("Index out of range.");
        &self.data[i]
    }
}

impl<T: Default + Clone + Copy> IndexMut<(usize, usize)> for Array2d<T> {
    fn index_mut<'a>(&'a mut self, pos: (usize, usize)) -> &'a mut T {
        let i = self.pos_to_i(pos).expect("Index out of range.");
        &mut self.data[i]
    }
}

impl<T: Default + Clone> IntoIterator for Array2d<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

