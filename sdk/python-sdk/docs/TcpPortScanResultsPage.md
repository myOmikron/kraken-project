# TcpPortScanResultsPage

Response containing paginated data

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**items** | [**List[SimpleTcpPortScanResult]**](SimpleTcpPortScanResult.md) | The page&#39;s items | 
**limit** | **int** | The limit this page was retrieved with | 
**offset** | **int** | The offset this page was retrieved with | 
**total** | **int** | The total number of items this page is a subset of | 

## Example

```python
from kraken_sdk.models.tcp_port_scan_results_page import TcpPortScanResultsPage

# TODO update the JSON string below
json = "{}"
# create an instance of TcpPortScanResultsPage from a JSON string
tcp_port_scan_results_page_instance = TcpPortScanResultsPage.from_json(json)
# print the JSON string representation of the object
print TcpPortScanResultsPage.to_json()

# convert the object into a dict
tcp_port_scan_results_page_dict = tcp_port_scan_results_page_instance.to_dict()
# create an instance of TcpPortScanResultsPage from a dict
tcp_port_scan_results_page_form_dict = tcp_port_scan_results_page.from_dict(tcp_port_scan_results_page_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


