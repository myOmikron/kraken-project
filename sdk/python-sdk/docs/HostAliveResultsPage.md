# HostAliveResultsPage

Response containing paginated data

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**items** | [**List[SimpleHostAliveResult]**](SimpleHostAliveResult.md) | The page&#39;s items | 
**limit** | **int** | The limit this page was retrieved with | 
**offset** | **int** | The offset this page was retrieved with | 
**total** | **int** | The total number of items this page is a subset of | 

## Example

```python
from kraken_sdk.models.host_alive_results_page import HostAliveResultsPage

# TODO update the JSON string below
json = "{}"
# create an instance of HostAliveResultsPage from a JSON string
host_alive_results_page_instance = HostAliveResultsPage.from_json(json)
# print the JSON string representation of the object
print HostAliveResultsPage.to_json()

# convert the object into a dict
host_alive_results_page_dict = host_alive_results_page_instance.to_dict()
# create an instance of HostAliveResultsPage from a dict
host_alive_results_page_form_dict = host_alive_results_page.from_dict(host_alive_results_page_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


