#!/usr/bin/env sh

echo "* updating language table"
curl --ssl -sL -o language.tab https://iso639-3.sil.org/sites/iso639-3/files/downloads/iso-639-3.tab || { echo "! failed"; exit; }

echo "* updating country json"
curl --ssl -sL -o country.json https://raw.githubusercontent.com/lukes/ISO-3166-Countries-with-Regional-Codes/master/all/all.json || { echo "! failed"; exit; }
