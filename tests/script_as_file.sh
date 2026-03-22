export PATH=${PATH}:$(pwd)/target/debug

TMP=$(mktemp)

printf 'set("OK")' > $TMP

RESULT="$(dop -e $TMP <<EOF
[1,2,3]
EOF
)"

TEST_NAME="Scripts can be passed as files"

if [ "${RESULT}" = '["OK","OK","OK"]' ]
then
    printf "✓ ${TEST_NAME}\n"

    exit 0
else
    printf "✖ ${TEST_NAME}\n"

    exit 1
fi
