# SearchEntry

Searched entry

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** |  | 
**created_at** | **datetime** |  | 
**finished_at** | **datetime** |  | [optional] 
**search_term** | **str** |  | 

## Example

```python
from kraken_sdk.models.search_entry import SearchEntry

# TODO update the JSON string below
json = "{}"
# create an instance of SearchEntry from a JSON string
search_entry_instance = SearchEntry.from_json(json)
# print the JSON string representation of the object
print SearchEntry.to_json()

# convert the object into a dict
search_entry_dict = search_entry_instance.to_dict()
# create an instance of SearchEntry from a dict
search_entry_form_dict = search_entry.from_dict(search_entry_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


