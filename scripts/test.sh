get_section () {
    awk '
BEGIN {
    IN_SECTION = 0;
}

/^#/  {
    if ($0 == sprintf("# %s", "'"${1}"'")) {
        IN_SECTION = 1;

        next;
    }
}

{
    if (IN_SECTION && $0 ~ /^#/) {
        exit;
    }

    if (IN_SECTION) {
        print $0;
    }
}
' | grep '.'
}

get_block () {
    awk '
BEGIN {
    RS="---\n";
    FOUND=0;
}

NR == '"$(($1+1))"' {
    print $0;

    FOUND=1;
}

END {
    if (FOUND == 1) {
        exit 0;
    }

    exit 1;
}
'
}

get_nth_section_name () {
    grep '^#' | head -n 1 | sed 's/^# \?//'
}

TEST_INPUT=$(mktemp)

find tests -type f | while read -r TEST_FILE
do
    CURRENT_BLOCK="0"

    echo $TEST_FILE | sed 's/tests\///'

    while true
    do
        if ! get_block ${CURRENT_BLOCK} < "${TEST_FILE}" > /dev/null
        then
            CURRENT_BLOCK="0"

            break;
        fi

        cat $TEST_FILE | get_block ${CURRENT_BLOCK} | get_section "INPUT" > $TEST_INPUT
        TEST_SCRIPT="$(cat $TEST_FILE | get_block ${CURRENT_BLOCK} | get_section "SCRIPT")"
        TEST_OPTIONS="$(cat $TEST_FILE | get_block ${CURRENT_BLOCK} | get_section "OPTIONS")"
        TEST_EXPECT="$(cat $TEST_FILE | get_block ${CURRENT_BLOCK} | get_section "EXPECT")"
        TEST_NAME="$(cat $TEST_FILE | get_block ${CURRENT_BLOCK} | get_nth_section_name 0)"

        if [ -z "${TEST_NAME}" ]
        then
            TEST_NAME="Nameless Test"
        fi

        if [ -n "${TEST_FILTER}" ] && ! printf "%s" "${TEST_NAME}" | grep -oq "${TEST_FILTER}" 2> /dev/null
        then
            printf "[SKIPPED] %s\n" "${TEST_NAME}"

            CURRENT_BLOCK="$(($CURRENT_BLOCK + 1))"

            continue
        fi

        TEST_RESULT="$(./target/debug/dop ${TEST_OPTIONS} "${TEST_SCRIPT}" < $TEST_INPUT)"

        if [ "${TEST_RESULT}" = "${TEST_EXPECT}" ]
        then
            printf "✓ %s\n" "${TEST_NAME}"
        else
            printf "✖ %s\nExpected: %s\nResult:   %s\n" "${TEST_NAME}" "${TEST_EXPECT}" "${TEST_RESULT}"
        fi

        CURRENT_BLOCK="$(($CURRENT_BLOCK + 1))"
    done
done
