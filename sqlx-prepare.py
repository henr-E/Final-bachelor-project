#!/usr/bin/env python3

# TODO: Convert this to a rust binary.
# TODO: Figure out a way to only clean compilation artefcats from this
# workspace. SQLX does this internally so it should be possible.

import os
import shutil
import subprocess

TARGET_DIR = "target"
SQLX_BUILD_DIR = "target/sqlx"
SQLX_PREPARE_DIR = ".sqlx"

subprocess.run(["cargo", "clean"])

os.makedirs(TARGET_DIR, exist_ok=True)

shutil.rmtree(SQLX_BUILD_DIR, ignore_errors=True)
shutil.rmtree(SQLX_PREPARE_DIR, ignore_errors=True)
os.mkdir(SQLX_BUILD_DIR)

os.environ["SQLX_OFFLINE"] = "false"
os.environ["SQLX_OFFLINE_DIR"] = SQLX_BUILD_DIR
subprocess.run(["cargo", "check", "--workspace"])

shutil.copytree(SQLX_BUILD_DIR, SQLX_PREPARE_DIR)
