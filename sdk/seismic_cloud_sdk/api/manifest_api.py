# coding: utf-8

"""
    Seismic Cloud Api

    The Seismic Cloud Api  # noqa: E501

    OpenAPI spec version: 0.0.1
    
    Generated by: https://github.com/swagger-api/swagger-codegen.git
"""


from __future__ import absolute_import

import re  # noqa: F401

# python 2 and python 3 compatibility library
import six

from seismic_cloud_sdk.api_client import ApiClient


class ManifestApi(object):
    """NOTE: This class is auto generated by the swagger code generator program.

    Do not edit the class manually.
    Ref: https://github.com/swagger-api/swagger-codegen
    """

    def __init__(self, api_client=None):
        if api_client is None:
            api_client = ApiClient()
        self.api_client = api_client

    def download_manifest(self, manifest_id, **kwargs):  # noqa: E501
        """download_manifest  # noqa: E501

        get manifest file  # noqa: E501
        This method makes a synchronous HTTP request by default. To make an
        asynchronous HTTP request, please pass async_req=True
        >>> thread = api.download_manifest(manifest_id, async_req=True)
        >>> result = thread.get()

        :param async_req bool
        :param str manifest_id: File ID (required)
        :return: StoreManifest
                 If the method is called asynchronously,
                 returns the request thread.
        """
        kwargs["_return_http_data_only"] = True
        if kwargs.get("async_req"):
            return self.download_manifest_with_http_info(
                manifest_id, **kwargs
            )  # noqa: E501
        else:
            (data) = self.download_manifest_with_http_info(
                manifest_id, **kwargs
            )  # noqa: E501
            return data

    def download_manifest_with_http_info(self, manifest_id, **kwargs):  # noqa: E501
        """download_manifest  # noqa: E501

        get manifest file  # noqa: E501
        This method makes a synchronous HTTP request by default. To make an
        asynchronous HTTP request, please pass async_req=True
        >>> thread = api.download_manifest_with_http_info(manifest_id, async_req=True)
        >>> result = thread.get()

        :param async_req bool
        :param str manifest_id: File ID (required)
        :return: StoreManifest
                 If the method is called asynchronously,
                 returns the request thread.
        """

        all_params = ["manifest_id"]  # noqa: E501
        all_params.append("async_req")
        all_params.append("_return_http_data_only")
        all_params.append("_preload_content")
        all_params.append("_request_timeout")

        params = locals()
        for key, val in six.iteritems(params["kwargs"]):
            if key not in all_params:
                raise TypeError(
                    "Got an unexpected keyword argument '%s'"
                    " to method download_manifest" % key
                )
            params[key] = val
        del params["kwargs"]
        # verify the required parameter 'manifest_id' is set
        if "manifest_id" not in params or params["manifest_id"] is None:
            raise ValueError(
                "Missing the required parameter `manifest_id` when calling `download_manifest`"
            )  # noqa: E501

        collection_formats = {}

        path_params = {}
        if "manifest_id" in params:
            path_params["manifest_id"] = params["manifest_id"]  # noqa: E501

        query_params = []

        header_params = {}

        form_params = []
        local_var_files = {}

        body_params = None
        # HTTP header `Accept`
        header_params["Accept"] = self.api_client.select_header_accept(
            ["application/octet-stream"]
        )  # noqa: E501

        # Authentication setting
        auth_settings = ["ApiKeyAuth"]  # noqa: E501

        return self.api_client.call_api(
            "manifest/{manifest_id}",
            "GET",
            path_params,
            query_params,
            header_params,
            body=body_params,
            post_params=form_params,
            files=local_var_files,
            response_type="StoreManifest",  # noqa: E501
            auth_settings=auth_settings,
            async_req=params.get("async_req"),
            _return_http_data_only=params.get("_return_http_data_only"),
            _preload_content=params.get("_preload_content", True),
            _request_timeout=params.get("_request_timeout"),
            collection_formats=collection_formats,
        )
