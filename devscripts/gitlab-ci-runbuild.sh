#!/bin/sh
set -e

# all error output to stdout, so shell tracing markers are kept in
# suitable ordering with collapsable section markers
exec 2>&1

FOOTER_ID="commit-$(git rev-parse --short HEAD)"
COLLAPSED_TITLE="$(git log -1 --oneline)"

# collapsable header
printf "\e[0Ksection_start:$(date +%s):${FOOTER_ID}[collapsed=true]\r\e[0K\e[1;33m${COLLAPSED_TITLE}\e[1;0m\n"
# trace, but not outside of collapsed section
set -x

IGNORED_ERROR=0
if ! "$@"; then
    ret=$?;
    case "$(git show --summary --format=format:%s)" in
        WIP*)
            IGNORED_ERROR=1
            ;;
        *)
            exit $ret
            ;;
    esac
fi

# stop traces before closing collapsed section
set +x
# collapsable footer
printf "\e[0Ksection_end:$(date +%s):${FOOTER_ID}\r\e[0K\n"

# make any ignored error visible outside of collapsed section
[ $IGNORED_ERROR = 0 ] || printf "\e[1;31mIgnoring failure for WIP commit\e[1;0m\n"
