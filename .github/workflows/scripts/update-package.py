import json 
import sys

package_version = sys.argv[1]

configuration = {
  "npmName": "@namada/indexer-client",
  "npmVersion": package_version,
  "httpUserAgent": ""
}

with open("swagger-codegen.json", 'w+', encoding='utf-8') as f:
    json.dump(configuration, f, ensure_ascii=False, indent=4)