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
    Linux   = if (arch == "x86-64") "rdocs-x86_64-unknown-linux-gnu.tar.xz" else "rdocs-aarch64-unknown-linux-gnu.tar.xz",
    Darwin  = if (arch == "x86-64") "rdocs-x86_64-apple-darwin.tar.xz" else "rdocs-aarch64-apple-darwin.tar.xz"
  )

  paste(RDOCS_URL_BASE, latest_release, binary, sep = "/")
}

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

generate_docs <- function(files, folder_name = "docs", gh_url = "") {
  files <- shQuote(normalizePath(files, mustWork = FALSE))
  gh_url <- shQuote(normalizePath(gh_url, mustWork = FALSE))
  # qmd_folder <- shQuote(file.path(folder_name, "docs_qmd", sep = "/"), mustWork = FALSE)
  quarto_check_path <- file.path(folder_name, ".quarto")

  if (!file.exists(RDOCS_PATH)) {
    cat("Downloading rdocs binary")
    download_rdocs()
  }

  # Create qmd files.
  system2(RDOCS_PATH, args = c("--files", files, "--docs-path", folder_name, "--gh-url", gh_url))
  # Create quarto project if it doesn't exist.
  if (!file.exists(quarto_check_path)) {
    system2("quarto", args = c("create", "project", "website", folder_name))
  }
  # Render.
  system2("quarto", args = c("render", folder_name))
}

generate_website <- function() {

}
