#include "pch.hpp"
#include "AppManager.h"
#include "AppManager.g.cpp"

namespace winrt::simbolmudah_ui::implementation
{
    AppManager::AppManager(Windows::Storage::ApplicationDataContainer const& localSettings)
        : localSettings{ localSettings }
    {
        const auto& values{ localSettings.Values() };

        if (values.HasKey(L"keyboardHookEnabled"))
        {
            this->hookEnabled = unbox_value<bool>(values.Lookup(L"keyboardHookEnabled"));
        }
        else
        {
            values.Insert(L"keyboardHookEnabled", box_value(this->hookEnabled));
        }

        if (values.HasKey(L"useHookPopup"))
        {
            this->useHookPopup = unbox_value<bool>(values.Lookup(L"useHookPopup"));
        }
        else
        {
            values.Insert(L"useHookPopup", box_value(this->useHookPopup));
        }

        if (values.HasKey(L"notifyIconEnabled"))
        {
            this->notifyIconEnabled = unbox_value<bool>(values.Lookup(L"notifyIconEnabled"));
        }
        else
        {
            values.Insert(L"notifyIconEnabled", box_value(this->notifyIconEnabled));
        }

        if (values.HasKey(L"mainWindowOpened"))
        {
            this->mainWindowOpened = unbox_value<bool>(values.Lookup(L"mainWindowOpened"));
        }
        else
        {
            values.Insert(L"mainWindowOpened", box_value(this->mainWindowOpened));
        }
    }

    void AppManager::HookEnabled(bool value)
    {
        if (this->hookEnabled != value)
        {
            this->hookEnabled = value;
            this->localSettings.Values().Insert(L"keyboardHookEnabled", box_value(this->hookEnabled));
            this->RaisePropertyChanged(L"HookEnabled");
        }
    }

    void AppManager::UseHookPopup(bool value)
    {
        if (this->useHookPopup != value)
        {
            this->useHookPopup = value;
            this->localSettings.Values().Insert(L"useHookPopup", box_value(this->useHookPopup));
            this->RaisePropertyChanged(L"UseHookPopup");
        }
    }

    void AppManager::NotifyIconEnabled(bool value)
    {
        if (this->notifyIconEnabled != value)
        {
            this->notifyIconEnabled = value;
            this->localSettings.Values().Insert(L"notifyIconEnabled", box_value(this->notifyIconEnabled));
            this->RaisePropertyChanged(L"NotifyIconEnabled");
        }
    }

    void AppManager::MainWindowOpened(bool value)
    {
        if (this->mainWindowOpened != value)
        {
            this->mainWindowOpened = value;
            this->localSettings.Values().Insert(L"mainWindowOpened", box_value(this->mainWindowOpened));
            this->RaisePropertyChanged(L"MainWindowOpened");
        }
    }
}
