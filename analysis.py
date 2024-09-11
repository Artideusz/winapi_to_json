import json

data = None

with open("./output.json", "r") as fh:
    data = json.loads(fh.read())


def most_functions(content):
    module_name = content[0]["module_name"]
    max_functions = len(content[0]["functions"])
    for module in content:
        if len(module["functions"]) > max_functions:
            module_name = module["module_name"]
            max_functions = len(module["functions"])
    return (module_name, max_functions)

def most_unique_datatypes(content):
    module_name = content[0]["module_name"]
    max_unique_datatypes = 0
    for module in content:
        datatypes = {}
        for function in module["functions"]:
            params = list(set(map(lambda v: v.split(" ")[0].replace("*", ""), function["params"])))
            for param in params:
                datatypes[param] = True
        if len(datatypes.keys()) > max_unique_datatypes:
            module_name = module["module_name"]
            max_unique_datatypes = len(datatypes.keys())
    return (module_name, max_unique_datatypes)



print(most_functions(data))
print(most_unique_datatypes(data))
print(len(data))

# {
#     "_EXAMPLE": {
#         "values": [
#             {
#                 "property_name": "EXAMPLE_STRUCT_INNER",
#                 "start": 0,
#                 "end": 8,
#             }
#         ]
#     },
# }