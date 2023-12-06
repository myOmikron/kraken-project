# GetAllLeechesResponse

The response that hold all leeches

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**leeches** | [**List[SimpleLeech]**](SimpleLeech.md) |  | 

## Example

```python
from kraken_sdk.models.get_all_leeches_response import GetAllLeechesResponse

# TODO update the JSON string below
json = "{}"
# create an instance of GetAllLeechesResponse from a JSON string
get_all_leeches_response_instance = GetAllLeechesResponse.from_json(json)
# print the JSON string representation of the object
print GetAllLeechesResponse.to_json()

# convert the object into a dict
get_all_leeches_response_dict = get_all_leeches_response_instance.to_dict()
# create an instance of GetAllLeechesResponse from a dict
get_all_leeches_response_form_dict = get_all_leeches_response.from_dict(get_all_leeches_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


