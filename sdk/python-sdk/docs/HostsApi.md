# kraken_sdk.HostsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_host**](HostsApi.md#create_host) | **POST** /api/v1/workspaces/{uuid}/hosts | Manually add a host
[**get_all_hosts**](HostsApi.md#get_all_hosts) | **POST** /api/v1/workspaces/{uuid}/hosts/all | Retrieve all hosts.
[**get_host**](HostsApi.md#get_host) | **GET** /api/v1/workspaces/{w_uuid}/hosts/{h_uuid} | Retrieve all information about a single host
[**update_host**](HostsApi.md#update_host) | **PUT** /api/v1/workspaces/{w_uuid}/hosts/{h_uuid} | Update a host


# **create_host**
> UuidResponse create_host(uuid, create_host_request)

Manually add a host

Manually add a host

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.create_host_request import CreateHostRequest
from kraken_sdk.models.uuid_response import UuidResponse
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.HostsApi(api_client)
    uuid = 'uuid_example' # str | 
    create_host_request = kraken_sdk.CreateHostRequest() # CreateHostRequest | 

    try:
        # Manually add a host
        api_response = await api_instance.create_host(uuid, create_host_request)
        print("The response of HostsApi->create_host:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling HostsApi->create_host: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **create_host_request** | [**CreateHostRequest**](CreateHostRequest.md)|  | 

### Return type

[**UuidResponse**](UuidResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Host was created |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_all_hosts**
> HostResultsPage get_all_hosts(uuid, get_all_hosts_query)

Retrieve all hosts.

Retrieve all hosts.  Hosts are created out of aggregating data or by user input. They represent a single host and can be created by providing an IP address

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.get_all_hosts_query import GetAllHostsQuery
from kraken_sdk.models.host_results_page import HostResultsPage
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.HostsApi(api_client)
    uuid = 'uuid_example' # str | 
    get_all_hosts_query = kraken_sdk.GetAllHostsQuery() # GetAllHostsQuery | 

    try:
        # Retrieve all hosts.
        api_response = await api_instance.get_all_hosts(uuid, get_all_hosts_query)
        print("The response of HostsApi->get_all_hosts:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling HostsApi->get_all_hosts: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **get_all_hosts_query** | [**GetAllHostsQuery**](GetAllHostsQuery.md)|  | 

### Return type

[**HostResultsPage**](HostResultsPage.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | All hosts in the workspace |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_host**
> FullHost get_host(w_uuid, h_uuid)

Retrieve all information about a single host

Retrieve all information about a single host

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.full_host import FullHost
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.HostsApi(api_client)
    w_uuid = 'w_uuid_example' # str | 
    h_uuid = 'h_uuid_example' # str | 

    try:
        # Retrieve all information about a single host
        api_response = await api_instance.get_host(w_uuid, h_uuid)
        print("The response of HostsApi->get_host:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling HostsApi->get_host: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **w_uuid** | **str**|  | 
 **h_uuid** | **str**|  | 

### Return type

[**FullHost**](FullHost.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Retrieved the selected host |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_host**
> update_host(w_uuid, h_uuid, update_host_request)

Update a host

Update a host  You must include at least on parameter

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.update_host_request import UpdateHostRequest
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.HostsApi(api_client)
    w_uuid = 'w_uuid_example' # str | 
    h_uuid = 'h_uuid_example' # str | 
    update_host_request = kraken_sdk.UpdateHostRequest() # UpdateHostRequest | 

    try:
        # Update a host
        await api_instance.update_host(w_uuid, h_uuid, update_host_request)
    except Exception as e:
        print("Exception when calling HostsApi->update_host: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **w_uuid** | **str**|  | 
 **h_uuid** | **str**|  | 
 **update_host_request** | [**UpdateHostRequest**](UpdateHostRequest.md)|  | 

### Return type

void (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Host was updated |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

