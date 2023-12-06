# CreateWordlistRequest

Arguments for creating a new wordlist

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** | The wordlist&#39;s name to be displayed select buttons | 
**description** | **str** | A description explaining the wordlist&#39;s intended use case | 
**path** | **str** | The file path the wordlist is deployed under on each leech | 

## Example

```python
from kraken_sdk.models.create_wordlist_request import CreateWordlistRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateWordlistRequest from a JSON string
create_wordlist_request_instance = CreateWordlistRequest.from_json(json)
# print the JSON string representation of the object
print CreateWordlistRequest.to_json()

# convert the object into a dict
create_wordlist_request_dict = create_wordlist_request_instance.to_dict()
# create an instance of CreateWordlistRequest from a dict
create_wordlist_request_form_dict = create_wordlist_request.from_dict(create_wordlist_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


