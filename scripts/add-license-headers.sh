#/bin/bash
set -eux

script_parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
header_file_path="$script_parent_path/../.maintain/AGPL-3.0-header.txt"

# We only want to run the subsitution on files which don't already have a Copyright header.
#
# When we do run the substitution then we first need to make sure that we add a new line after the
# header, and then make sure that we print the header before the old "first line" of the file (with
# `N`).
#
# NOTE: This sometimes removes the last line of a file and I'm not sure why. So double check before
# committing any changes.
rg --type rust "Copyright" --files-without-match | xargs sed -i '' -e "1r $header_file_path" -e "1s|^|\n|" -e "N"
