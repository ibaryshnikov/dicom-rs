# DICOM-rs 

[![Build Status](https://travis-ci.org/Enet4/dicom-rs.svg?branch=master)](https://travis-ci.org/Enet4/dicom-rs)

An efficient and practical base library for DICOM compliant systems.

At its core, this library is a pure Rust implementation of the DICOM representation format,
allowing users to read and write DICOM objects over files and other sources, while
remaining intrinsically fast and safe to use.

This project is a WIP. No crate has been published online to respect the fact that
the project does not yet meet the minimum features required to be usable.

## Components

 - [`core`](core) represents all of the base traits, data structures and functions for
 reading and writing DICOM content.
 - [`dictionary_builder`](dictionary_builder) is a Rust application that generates Rust
   code for a DICOM standard dictionary using entries from the web.
