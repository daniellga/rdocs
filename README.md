## rdocs: Simple Quarto documentation for R packages 

Allows to create quarto markdown documentation (.qmd) from `///` and `###` tokens in text files. Documentation can be created, for example, from `.R`, `.rs` and 
`.cpp` files. \
The package will search for these tokens in the files and generate a quarto website.

## Instalation

Install [Quarto](https://quarto.org/) and add it to PATH.

Install the package from github:
```r
remotes::install_github("daniellga/rdocs", subdir = "rdocs")
```


## Usage

An example is showed below, where 2 distinct functions will be documented on the same section, named HFft. 
The first line of the `///` block is important, since the name of the variable will be used to group functions into the same one-worded section. This is useful, for example, when working using an OOP approach. \
The user is free to create its headings, line breaks and make use of all other markdown utilities. \
All R code identified by "```r" will be evaluated in a separate R session. If you don't want a certain part of the code to be run, it will be needed to comment it or avoid the "```r" representation. If you don't want to evaluate the code at all, use `run_examples = FALSE`, which is the default. \

`testscript.rs`
```r
/// HFft
/// ## fft
///
/// `fft() -> HArray` \
///
/// Computes the fast fourier transform of the `HArray`. \
///
/// #### Returns
///
/// An `HArray`. \
///
/// #### Examples
///
/// ```r
/// arr = array(c(1,2,3,4,5,6,7,8,9,10,11,12), c(3,4))
/// dtype = HDataType$float32
/// HArray$new_from_values(arr, dtype)
/// HFft$fft(harray)
/// ```
///
fn fft(harray: &HArray) -> HArray {
    HArray(harray.0.fft())
}

/// HFft
/// ## fft_mut
///
/// `fft_mut()` \
///
/// Computes the fast fourier transform of the `HArray`. \
/// The operation is done in-place. \

/// #### Examples
///
/// ```r
/// arr = array(c(1,2,3,4,5,6,7,8,9,10,11,12), c(3,4))
/// dtype = HDataType$float32
/// HArray$new_from_values(arr, dtype)
/// HFft$fft_mut(harray)
/// ```
///
fn fft_mut(harray: &mut HArray) {
    let inner_mut = harray.get_inner_mut();
    inner_mut.fft_mut()
}
```

```r
rdocs::generate_docs("./testscript.R")
```

The website will be generated on the current working directory.

This library's [documentation](https://daniellga.github.io/rdocs/) was created using rdocs.


