# SearchTypeOneOf3


## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**var_or** | [**List[SearchType]**](SearchType.md) | Add multiple [SearchType]s with an OR | 

## Example

```python
from kraken_sdk.models.search_type_one_of3 import SearchTypeOneOf3

# TODO update the JSON string below
json = "{}"
# create an instance of SearchTypeOneOf3 from a JSON string
search_type_one_of3_instance = SearchTypeOneOf3.from_json(json)
# print the JSON string representation of the object
print SearchTypeOneOf3.to_json()

# convert the object into a dict
search_type_one_of3_dict = search_type_one_of3_instance.to_dict()
# create an instance of SearchTypeOneOf3 from a dict
search_type_one_of3_form_dict = search_type_one_of3.from_dict(search_type_one_of3_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


