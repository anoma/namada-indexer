import json 
import sys

package_json_path = sys.argv[1]
package_version = sys.argv[2]

package_json = json.load(open(package_json_path))

package_json['name'] = "namada-indexer-client"
package_json['version'] = package_version
package_json['description'] = "Set of API to interact with a namada indexer."
package_json['license'] = " GPL-3.0 license"

with open(package_json_path, 'w', encoding='utf-8') as f:
    json.dump(package_json, f, ensure_ascii=False, indent=4)