# SearchTypeOneOf4


## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**var_and** | [**List[SearchType]**](SearchType.md) | Add multiple [SearchType]s with an AND | 

## Example

```python
from kraken_sdk.models.search_type_one_of4 import SearchTypeOneOf4

# TODO update the JSON string below
json = "{}"
# create an instance of SearchTypeOneOf4 from a JSON string
search_type_one_of4_instance = SearchTypeOneOf4.from_json(json)
# print the JSON string representation of the object
print SearchTypeOneOf4.to_json()

# convert the object into a dict
search_type_one_of4_dict = search_type_one_of4_instance.to_dict()
# create an instance of SearchTypeOneOf4 from a dict
search_type_one_of4_form_dict = search_type_one_of4.from_dict(search_type_one_of4_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


