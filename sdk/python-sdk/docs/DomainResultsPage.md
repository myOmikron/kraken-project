# DomainResultsPage

Response containing paginated data

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**items** | [**List[FullDomain]**](FullDomain.md) | The page&#39;s items | 
**limit** | **int** | The limit this page was retrieved with | 
**offset** | **int** | The offset this page was retrieved with | 
**total** | **int** | The total number of items this page is a subset of | 

## Example

```python
from kraken_sdk.models.domain_results_page import DomainResultsPage

# TODO update the JSON string below
json = "{}"
# create an instance of DomainResultsPage from a JSON string
domain_results_page_instance = DomainResultsPage.from_json(json)
# print the JSON string representation of the object
print DomainResultsPage.to_json()

# convert the object into a dict
domain_results_page_dict = domain_results_page_instance.to_dict()
# create an instance of DomainResultsPage from a dict
domain_results_page_form_dict = domain_results_page.from_dict(domain_results_page_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


