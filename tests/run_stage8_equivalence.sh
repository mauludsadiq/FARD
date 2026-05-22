#!/usr/bin/env bash
set -euo pipefail

FILES=(
  spec/tmp/g54_g60_kitchen_sink.fard
  spec/tmp/g54_g60_kitchen_sink_0.fard
  spec/tmp/g54_g60_kitchen_sink_1.fard
  spec/tmp/g54_g60_kitchen_sink_2.fard
  spec/tmp/g54_g60_kitchen_sink_3.fard
  spec/tmp/g54_g60_kitchen_sink_4.fard
  spec/tmp/g61_g65_kitchen_sink_0.fard
  spec/tmp/g61_g65_kitchen_sink_1.fard
  spec/tmp/g61_g65_kitchen_sink_2.fard
  spec/tmp/g61_g65_kitchen_sink_3.fard
  spec/tmp/g61_g65_kitchen_sink_4.fard
  spec/tmp/g71_g75_kitchen_sink_0.fard
  spec/tmp/g71_g75_kitchen_sink_1.fard
  spec/tmp/g71_g75_kitchen_sink_2.fard
  spec/tmp/g71_g75_kitchen_sink_3.fard
  spec/tmp/g71_g75_kitchen_sink_4.fard
  spec/tmp/g71_g75_kitchen_sink_5.fard
  spec/tmp/g71_g75_kitchen_sink_6.fard
)

rm -rf /tmp/fard_stage8_equiv
mkdir -p /tmp/fard_stage8_equiv

for f in "${FILES[@]}"; do
  name="$(echo "$f" | tr '/.' '__')"
  normal="/tmp/fard_stage8_equiv/${name}_normal"
  evald="/tmp/fard_stage8_equiv/${name}_eval"

  echo "===== $f"
  fardrun run --program "$f" --out "$normal" >/tmp/fard_stage8_equiv/normal.log 2>&1
  fardrun run --program "$f" --out "$evald" --fard-eval >/tmp/fard_stage8_equiv/eval.log 2>&1

  diff -u "$normal/result.json" "$evald/result.json"
done

echo "stage8 equivalence passed"
