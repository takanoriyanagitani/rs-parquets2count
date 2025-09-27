#!/bin/bash

gencsv(){
	cbasename=$1

	mkdir -p ./sample.d

	cname="./sample.d/${cbasename}"

	exec 11>&1
	exec 1>"${cname}"

	printf 'timestamp,severity,body\n'
	printf '%s,%s,"%s"\n' "$( date )" "INFO" 'hello, world'
	printf '%s,%s,"%s"\n' "$( date )" "INFO" 'apt update done'
	printf '%s,%s,"%s"\n' "$( date )" "WARN" 'apt update failure'

	exec 1>&!!
	exec 11>&-
}

geninput(){
	echo generating input files...

	gencsv "tmp0.csv"
	gencsv "tmp1.csv"

	which rs-csv2parquet | fgrep -q rs-csv2parquet || exec sh -c '
		echo rs-csv2parquet missing.
		echo you can install it using cargo install.
		exit 1
	'

	rs-csv2parquet \
		--input-csv-filename './sample.d/tmp0.csv' \
		--output-parquet-filename './sample.d/in0.parquet' \
		--has-header \
		--same-column-count

	rs-csv2parquet \
		--input-csv-filename './sample.d/tmp1.csv' \
		--output-parquet-filename './sample.d/in1.parquet' \
		--has-header \
		--same-column-count

}

test -f './sample.d/in0.parquet' || geninput
test -f './sample.d/in1.parquet' || geninput

find ./sample.d -type f -name '*.parquet' |
	./parquets2count
