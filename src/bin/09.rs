#![feature(iter_array_chunks)]

use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt::Display;
use std::iter;
use std::num::ParseIntError;
use std::str::FromStr;

use derive_more::derive::Display;
use derive_more::derive::Error;
use derive_more::derive::From;

advent_of_code::solution!(9);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BlockType {
    File { file_id: usize },
    Free(),
}

impl BlockType {
    pub fn is_free(&self) -> bool {
        matches!(self, Self::Free())
    }

    pub fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    pub fn file_id(&self) -> Option<usize> {
        if let Self::File { file_id } = self {
            Some(*file_id)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DiskBlock {
    pub block_type: BlockType,
}

#[derive(Debug, Clone, Copy)]
pub struct DiskChunk {
    pub blocks: usize,
    pub chunk_type: BlockType,
}

impl DiskChunk {
    // chunk will be exploded into chunks with block size 1
    pub fn explode<'a>(&'a self) -> impl Iterator<Item = DiskBlock> + use<'a> {
        iter::repeat(false)
            .map(|_| DiskBlock {
                block_type: self.chunk_type,
            })
            .take(self.blocks)
    }
}

impl Display for DiskChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.chunk_type {
            BlockType::File { file_id } => {
                for _ in 0..self.blocks {
                    write!(f, "{file_id}")?;
                    if file_id >= 10 {
                        write!(f, " ")?;
                    }
                }
            }
            BlockType::Free() => {
                // display free space
                for _ in 0..self.blocks {
                    write!(f, ".")?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DiskMap {
    chunks: VecDeque<DiskChunk>,
}

impl DiskMap {
    pub fn is_fragmentated(&self) -> bool {
        // invariant: after first free chunk there is no other file chunk

        let mut chunks = self.chunks.iter();
        let _ = chunks
            .by_ref()
            .take_while(|chunk| chunk.chunk_type.is_file())
            .collect::<Vec<_>>();

        chunks.any(|chunk| chunk.chunk_type.is_file())
    }

    pub fn defrag(&mut self) {
        loop {
            // find free chunk to move data from the back
            let Some((free_pointer, mut free_chunk)) = self
                .chunks
                .iter()
                .enumerate()
                .find(|(_, disk_chunk)| disk_chunk.chunk_type.is_free() && disk_chunk.blocks > 0)
                .map(|(p, c)| (p, *c))
            else {
                break;
            };

            // get data block from the back
            let Some((file_pointer, mut file_chunk)) = self
                .chunks
                .iter()
                .enumerate()
                .rev()
                .find(|(_, disk_chunk)| disk_chunk.chunk_type.is_file() && disk_chunk.blocks > 0)
                .map(|(p, c)| (p, *c))
            else {
                break;
            };

            //println!("Free: {free_chunk} at {free_chunk}");
            //println!("File: {file_chunk} at {file_pointer}");

            // (partially) move file block into free block
            {
                let file_blocks = file_chunk.blocks;
                let free_blocks = free_chunk.blocks;
                // max amount we can fit of the file into free block
                let delta = usize::min(file_blocks, free_blocks);
                //println!("Move {delta} of {:?}", file_chunk.chunk_type);

                file_chunk.blocks -= delta;
                free_chunk.blocks -= delta;
                //
                // write back changes chunks
                // NOTE: it doesn't matter if we leave behind 0 blocks because of final cleanup
                self.chunks[free_pointer] = free_chunk;
                self.chunks[file_pointer] = file_chunk;

                // create new file block in front of free block for moved content of file
                self.chunks.insert(
                    free_pointer,
                    DiskChunk {
                        blocks: delta,
                        chunk_type: file_chunk.chunk_type,
                    },
                );
            }
            //println!("Self: {self}");
        }

        // cleanup unneeded chunks
        self.chunks.retain(|disk_chunk| disk_chunk.blocks > 0);

        assert!(
            !self.is_fragmentated(),
            "should be defragmentated after defrag"
        );
    }

    pub fn defrag_whole_file(&mut self) {
        // FIXME:collapse free chunks?

        let mut checked_files = HashSet::new();
        loop {
            // get data block from the back
            let Some((file_pointer, mut file_chunk)) = self
                .chunks
                .iter()
                .enumerate()
                .rev()
                .find(|(_, disk_chunk)| {
                    disk_chunk
                        .chunk_type
                        .file_id()
                        .is_some_and(|fid| !checked_files.contains(&fid))
                        && disk_chunk.blocks > 0
                })
                .map(|(p, c)| (p, *c))
            else {
                break;
            };
            //println!("File: {file_chunk} at {file_pointer}");
            checked_files.insert(file_chunk.chunk_type.file_id().unwrap());

            // find free chunk to would fit our data
            let Some((free_pointer, mut free_chunk)) = self
                .chunks
                .iter()
                // only search in front of file
                .take(file_pointer.saturating_sub(1))
                .enumerate()
                .find(|(_, disk_chunk)| {
                    disk_chunk.chunk_type.is_free() && disk_chunk.blocks >= file_chunk.blocks
                })
                .map(|(p, c)| (p, *c))
            else {
                // no fitting place found :(
                continue;
            };
            //println!("Free: {free_chunk} at {free_chunk}");

            // (partially) move file block into free block
            {
                let file_blocks = file_chunk.blocks;
                let free_blocks = free_chunk.blocks;
                // max amount we can fit of the file into free block
                let delta = usize::min(file_blocks, free_blocks);
                //println!("Move {delta} of {:?}", file_chunk.chunk_type);

                file_chunk.blocks -= delta;
                free_chunk.blocks -= delta;
                //
                // write back changes chunks
                // NOTE: it doesn't matter if we leave behind 0 blocks because of final cleanup
                self.chunks[free_pointer] = free_chunk;
                self.chunks[file_pointer] = file_chunk;

                // create new file block in front of free block for moved content of file
                self.chunks.insert(
                    free_pointer,
                    DiskChunk {
                        blocks: delta,
                        chunk_type: file_chunk.chunk_type,
                    },
                );
                // create empty whole where file left
                self.chunks.insert(
                    file_pointer + 1,
                    DiskChunk {
                        blocks: delta,
                        chunk_type: free_chunk.chunk_type,
                    },
                );
            }
            //println!("Self: {self}");
        }

        // cleanup unneeded chunks
        self.chunks.retain(|disk_chunk| disk_chunk.blocks > 0);
    }

    // returns one-sized chunks
    pub fn checksum(&self) -> usize {
        self.chunks
            .iter()
            .flat_map(DiskChunk::explode)
            .enumerate()
            .fold(0, |partial_sum, (pos, block)| {
                partial_sum + pos * block.block_type.file_id().unwrap_or(0)
            })
    }
}

impl Display for DiskMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for disk_chunk in self.chunks.iter() {
            write!(f, "{}", disk_chunk)?;
        }

        Ok(())
    }
}

#[derive(Debug, Error, Display, From)]
pub enum ParseDiskMapError {
    ParseIntError(ParseIntError),
    OddNumbers(),
}
impl FromStr for DiskMap {
    type Err = ParseDiskMapError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut disk_chunks = s.lines().next().unwrap().chars().array_chunks::<2>();

        let mut chunks: VecDeque<_> = disk_chunks
            .by_ref()
            .enumerate()
            .flat_map(|(file_id, disk_chunk)| {
                let file_size = disk_chunk[0].to_digit(10).unwrap() as usize;
                let free_size = disk_chunk[1].to_digit(10).unwrap() as usize;

                [
                    DiskChunk {
                        blocks: file_size,
                        chunk_type: BlockType::File { file_id },
                    },
                    DiskChunk {
                        blocks: free_size,
                        chunk_type: BlockType::Free(),
                    },
                ]
            })
            .collect();

        if let Some(mut remainder) = disk_chunks.into_remainder() {
            let file_size = remainder.next().unwrap().to_digit(10).unwrap() as usize;
            chunks.push_back(DiskChunk {
                blocks: file_size,
                chunk_type: BlockType::File {
                    file_id: chunks.len() / 2,
                },
            })
        }

        Ok(Self { chunks })
    }
}

pub fn part_one(input: &str) -> Option<usize> {
    let mut disk_map: DiskMap = input.parse().unwrap();
    //println!("Fragmanted Disk: {}", disk_map);

    disk_map.defrag();
    //println!("Defragmanted Disk: {}", disk_map);

    Some(disk_map.checksum())
}

pub fn part_two(input: &str) -> Option<usize> {
    let mut disk_map: DiskMap = input.parse().unwrap();
    //println!("Fragmanted Disk: {}", disk_map);

    disk_map.defrag_whole_file();
    //println!("Defragmanted Disk: {}", disk_map);

    Some(disk_map.checksum())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(1928));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(2858));
    }
}
