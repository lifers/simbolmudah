#include "pch.hpp"
#include "AppManager.h"
#include "AppManager.g.cpp"

namespace winrt::simbolmudah_ui::implementation
{
    AppManager::AppManager(Windows::Storage::ApplicationDataContainer const& localSettings)
        : main_thread{ apartment_context() }, localSettings{ localSettings }
    {
        const auto& values{ localSettings.Values() };

        if (values.HasKey(L"keyboardHookEnabled"))
        {
            this->m_HookEnabled = unbox_value<bool>(values.Lookup(L"keyboardHookEnabled"));
        }
        else
        {
            values.Insert(L"keyboardHookEnabled", box_value(this->m_HookEnabled));
        }

        if (values.HasKey(L"useHookPopup"))
        {
            this->m_UseHookPopup = unbox_value<bool>(values.Lookup(L"useHookPopup"));
        }
        else
        {
            values.Insert(L"useHookPopup", box_value(this->m_UseHookPopup));
        }

        if (values.HasKey(L"notifyIconEnabled"))
        {
            this->m_NotifyIconEnabled = unbox_value<bool>(values.Lookup(L"notifyIconEnabled"));
        }
        else
        {
            values.Insert(L"notifyIconEnabled", box_value(this->m_NotifyIconEnabled));
        }

        if (values.HasKey(L"mainWindowOpened"))
        {
            this->m_MainWindowOpened = unbox_value<bool>(values.Lookup(L"mainWindowOpened"));
        }
        else
        {
            values.Insert(L"mainWindowOpened", box_value(this->m_MainWindowOpened));
        }
    }

    fire_and_forget AppManager::SaveSettings(simbolmudah_ui::SettingsObject settings)
    {
        co_await this->main_thread;

        const auto& values{ this->localSettings.Values() };

        if (settings.HookEnabled != this->m_HookEnabled)
        {
            this->m_HookEnabled = settings.HookEnabled;
            values.Insert(L"keyboardHookEnabled", box_value(this->m_HookEnabled));
        }

        if (settings.UseHookPopup != this->m_UseHookPopup)
        {
            this->m_UseHookPopup = settings.UseHookPopup;
            values.Insert(L"useHookPopup", box_value(this->m_UseHookPopup));
        }

        if (settings.NotifyIconEnabled != this->m_NotifyIconEnabled)
        {
            this->m_NotifyIconEnabled = settings.NotifyIconEnabled;
            values.Insert(L"notifyIconEnabled", box_value(this->m_NotifyIconEnabled));
        }

        if (settings.MainWindowOpened != this->m_MainWindowOpened)
        {
            this->m_MainWindowOpened = settings.MainWindowOpened;
            values.Insert(L"mainWindowOpened", box_value(this->m_MainWindowOpened));
        }

        this->RaisePropertyChanged(L"");
    }
}
