RDOCS_URL_BASE <- "https://github.com/daniellga/rdocs/releases/download"

RDOCS_NAME <- "rdocs"

RDOCS_CACHE_DIR <- normalizePath(tools::R_user_dir("rdocs", "cache"), mustWork = FALSE)

RDOCS_PATH <- {
  if (Sys.info()[["sysname"]] == "Windows") {
    file.path(RDOCS_CACHE_DIR, paste0(RDOCS_NAME, ".exe"))
  } else {
    file.path(RDOCS_CACHE_DIR, RDOCS_NAME)
  }
}

latest_release <- jsonlite::read_json("https://api.github.com/repos/daniellga/rdocs/releases/latest")[["tag_name"]]

get_download_url <- function() {
  os <- Sys.info()[["sysname"]]
  arch <- Sys.info()[["machine"]]

  binary <- switch(os,
    Windows = "rdocs-x86_64-pc-windows-msvc.zip",
    Linux   = "rdocs-x86_64-unknown-linux-gnu.tar.xz",
    Darwin  = if (arch == "x86-64") "rdocs-x86_64-apple-darwin.tar.xz" else "rdocs-aarch64-apple-darwin.tar.xz"
  )

  paste(RDOCS_URL_BASE, latest_release, binary, sep = "/")
}

### Functions
### ## download_rdocs
###
### `download_rdocs()` \
###
### Forces the update of the cached binary. \
### Use this when the package version has been updated and a new updated release is available. \
###
### #### Examples
###
### ```r
### rdocs::download_rdocs()
### ```
###
download_rdocs <- function() {
  download_tmp_dir <- tempfile()
  extract_tmp_dir <- tempfile()
  on.exit(unlink(download_tmp_dir, recursive = TRUE, force = TRUE), add = TRUE)
  on.exit(unlink(extract_tmp_dir, recursive = TRUE, force = TRUE), add = TRUE)

  # download
  dir.create(download_tmp_dir)
  download_url <- get_download_url()
  archive_file <- file.path(download_tmp_dir, basename(download_url))
  download.file(download_url, destfile = archive_file, mode = "wb")

  # extract and copy.
  if (Sys.info()[["sysname"]] == "Windows") {
    utils::unzip(archive_file, exdir = extract_tmp_dir)
    file.copy(file.path(extract_tmp_dir, "rdocs.exe"), RDOCS_PATH, overwrite = TRUE)
  } else {
    utils::untar(archive_file, exdir = extract_tmp_dir, extras = "--strip-components=1")
    file.copy(file.path(extract_tmp_dir, "rdocs"), RDOCS_PATH, overwrite = TRUE)
  }

  invisible(NULL)
}

### Functions
### ## generate_docs
###
### `generate_docs(files, folder_name = "docs", gh_url = "", run_examples = FALSE)` \
###
### Generate a quarto website from lines starting with `###` or `///` that are right above function declarations. \
### The website folder is created in the current working directory. \
### The first line of the block is important, since the name of the variable will be used to group functions into the
### same one-worded section. This is useful, for example, when working using an OOP approach. For now, to avoid any
### bugs, it is important that all grouped functions are on the same file. \
### The user is free to create its headings, line breaks and make use of all other markdown utilities. \
### All R code identified by r code markdown blocks will be evaluated in a separate R session. If you don't want a certain part of the
### code to be run, it will be needed to comment it or avoid the code block representation. If you don't want to evaluate
### the code at all, use `run_examples = FALSE`, which is the default. \
###
### #### Arguments
###
### * `files` \
### The files that will be used to create the `.qmd` documentation.
### * `folder_name` \
### Name of the folder which will store the website. \
### * `gh_url` \
### A github url indicating where the documented files will be stored. It will create a link to the source code for each
### function. The default value will not create such links. \
### * `run_examples` \
### If `TRUE`, All R code identified in an r code block will be evaluated in a separate R session. If you don't want a certain part
### of the code to be run, it will be needed to comment it or avoid the r code representation in markdown. \
###
###
### #### Examples
###
### ```r
### rdocs::generate_docs(files = "./rdocs/R/main.R", folder_name = "docs", gh_url = "https://github.com/daniellga/rdocs/tree/main/rdocs/R", run_examples = FALSE)
### ```
###
generate_docs <- function(files, folder_name = "docs", gh_url = "", run_examples = FALSE) {
  files <- shQuote(files)
  gh_url <- shQuote(gh_url)
  folder_name <- shQuote(folder_name)

  if (!file.exists(RDOCS_PATH)) {
    cat("Downloading rdocs binary.")
    download_rdocs()
  }

  run_examples <- if (isTRUE(run_examples)) "--run-examples" else NULL

  # Create qmd files and build and render quarto project.
  system2(RDOCS_PATH, args = c("--files", files, "--folder-name", folder_name, "--gh-url", gh_url, run_examples))

  cat("Documentation has been built.")

  invisible(NULL)
}
