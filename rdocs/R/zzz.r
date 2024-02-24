.onLoad <- function(libname, pkgname) {
  dir.create(RDOCS_CACHE_DIR, recursive = TRUE, showWarnings = FALSE)

  invisible(NULL)
}
