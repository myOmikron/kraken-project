# SearchType

A specific search type

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**simple** | **str** | Search for a simple pattern | 
**exact** | **str** | Search for an exact pattern | 
**regex** | **str** | A regex search pattern | 
**var_or** | [**List[SearchType]**](SearchType.md) | Add multiple [SearchType]s with an OR | 
**var_and** | [**List[SearchType]**](SearchType.md) | Add multiple [SearchType]s with an AND | 

## Example

```python
from kraken_sdk.models.search_type import SearchType

# TODO update the JSON string below
json = "{}"
# create an instance of SearchType from a JSON string
search_type_instance = SearchType.from_json(json)
# print the JSON string representation of the object
print SearchType.to_json()

# convert the object into a dict
search_type_dict = search_type_instance.to_dict()
# create an instance of SearchType from a dict
search_type_form_dict = search_type.from_dict(search_type_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


