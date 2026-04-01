set -e

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
' | sed '/./,$!d' | sed ':a;/^\n*$/{$d;N;ba}'
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

export PATH=${PATH}:$(pwd)/target/debug

find tests -type f | grep -v \.sh$ | sort | while read -r TEST_FILE
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
        TEST_SCRIPT_ONCE="$(cat $TEST_FILE | get_block ${CURRENT_BLOCK} | get_section "SCRIPT_ONCE")"
        TEST_OPTIONS="$(cat $TEST_FILE | get_block ${CURRENT_BLOCK} | get_section "OPTIONS")"
        TEST_NAME="$(cat $TEST_FILE | get_block ${CURRENT_BLOCK} | get_nth_section_name 0)"
        TEST_EXPECT="$(cat $TEST_FILE | get_block ${CURRENT_BLOCK} | get_section "EXPECT")"
        TEST_EXPECTS_ERROR="false"
        if [ -z "${TEST_EXPECT}" ]
        then
            TEST_EXPECT="$(cat $TEST_FILE | get_block ${CURRENT_BLOCK} | get_section "EXPECT_ERROR")"
            TEST_EXPECTS_ERROR="true"
        fi

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

        set +e
        if [ -n "${TEST_SCRIPT_ONCE}" ]
        then
            TEST_RESULT="$(./target/debug/dop ${TEST_OPTIONS} -E "${TEST_SCRIPT_ONCE}" < $TEST_INPUT 2>&1)"
        elif [ -z "${TEST_SCRIPT}" ]
        then
            TEST_RESULT="$(./target/debug/dop ${TEST_OPTIONS} < $TEST_INPUT 2>&1)"
        else
            TEST_RESULT="$(./target/debug/dop ${TEST_OPTIONS} -e "${TEST_SCRIPT}" < $TEST_INPUT 2>&1)"
        fi
        TEST_RESULT_STATUS_CODE="${?}"
        set -e

        if [ "${TEST_EXPECTS_ERROR}" = "true" ] && [ "${TEST_RESULT_STATUS_CODE}" = "0" ]
        then
            printf "✖ %s\nExpected status code to be non-zero.\n" "${TEST_NAME}"

            exit 1
        fi

        if [ "${TEST_RESULT}" = "${TEST_EXPECT}" ]
        then
            printf "✓ %s\n" "${TEST_NAME}"
        else
            printf "✖ %s\nExpected: %s\nResult:   %s\n" "${TEST_NAME}" "${TEST_EXPECT}" "${TEST_RESULT}"

            exit 1
        fi

        CURRENT_BLOCK="$(($CURRENT_BLOCK + 1))"
    done
done

echo "---"

sh tests/*.sh
