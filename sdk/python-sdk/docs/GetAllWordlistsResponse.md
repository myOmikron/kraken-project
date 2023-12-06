# GetAllWordlistsResponse

Response containing all wordlists

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**wordlists** | [**List[SimpleWordlist]**](SimpleWordlist.md) | List of all wordlists | 

## Example

```python
from kraken_sdk.models.get_all_wordlists_response import GetAllWordlistsResponse

# TODO update the JSON string below
json = "{}"
# create an instance of GetAllWordlistsResponse from a JSON string
get_all_wordlists_response_instance = GetAllWordlistsResponse.from_json(json)
# print the JSON string representation of the object
print GetAllWordlistsResponse.to_json()

# convert the object into a dict
get_all_wordlists_response_dict = get_all_wordlists_response_instance.to_dict()
# create an instance of GetAllWordlistsResponse from a dict
get_all_wordlists_response_form_dict = get_all_wordlists_response.from_dict(get_all_wordlists_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


