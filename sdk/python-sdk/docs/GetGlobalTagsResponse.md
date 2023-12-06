# GetGlobalTagsResponse

The response to a request to retrieve all global tags

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**global_tags** | [**List[FullGlobalTag]**](FullGlobalTag.md) |  | 

## Example

```python
from kraken_sdk.models.get_global_tags_response import GetGlobalTagsResponse

# TODO update the JSON string below
json = "{}"
# create an instance of GetGlobalTagsResponse from a JSON string
get_global_tags_response_instance = GetGlobalTagsResponse.from_json(json)
# print the JSON string representation of the object
print GetGlobalTagsResponse.to_json()

# convert the object into a dict
get_global_tags_response_dict = get_global_tags_response_instance.to_dict()
# create an instance of GetGlobalTagsResponse from a dict
get_global_tags_response_form_dict = get_global_tags_response.from_dict(get_global_tags_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


