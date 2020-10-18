fdedup - File Deduplicator
==========================

`fdedup` is a cmdline file deduplication program for unix filesystems. `fdedup`
is highly inspired by `rdfind` but aims to better support deduplication of
backups that rely on hardlinks.

When `fdedup` is run it will first scan user supplied paths and collect all
found files. Then a duplicate file search algorithm similar to `rdfind` will be
executed in order to determine which files are identical (based on file hash).
At the end, `fdedup` can print the summary of found duplicate files, or perform
the actual deduplication, replacing duplicates by either hardlinks or symlinks.


Installation
------------

`fdedup` is written in ``rust`` and relies on a ``cargo`` package manager. You
can use either ``cargo`` commands to build and install `fdedup` or use provided
``Makefile``

::

    make install

You can control installation paths via ``PREFIX``, ``DESTDIR`` variables.


Usage Examples
--------------

Search for duplicates in home directory and save results to ~/dups.txt

::

    $ fdedup --output ~/dups.txt ~/

Find duplicates and replace them by symlinks in home directory ignoring
noncritical I/O errors

::

    $ fdedup --action symlink --sloppy ~/

Deduplicate backup drive ``/mnt/backup``

::

    $ fdedup --action hardlink /mnt/backup


Performance
-----------

I have directly compared performance of `fdedup` to `rdfind`
in ``-removeidentinode true`` mode on my backup HDD with dropped caches.
Both `fdedup` to `rdfind` display exactly the same runtime (~20min) within
statistical error and `fdedup` uses slightly less RAM overall.  However, in the
``-removeidentinode true`` mode `rdfind` fails to handle the existing hardlinks
correctly. Running `rdfind` in the ``-removeidentinode false`` mode makes it
handle hardlinks properly, but increases its runtime by a factor of 6, making
`fdedup` a clear winner in this case.


Disclaimer
----------

This is an **alpha** software. I have tested it only on the ``ext4``
filesystem.

Use at your own risk.

