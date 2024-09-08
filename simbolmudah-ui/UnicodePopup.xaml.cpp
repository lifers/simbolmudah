#include "pch.hpp"
#include "UnicodePopup.xaml.h"
#if __has_include("UnicodePopup.g.cpp")
#include "UnicodePopup.g.cpp"
#endif

namespace
{
    constinit auto hexChars{ L"0123456789ABCDEFabcdef" };

    constexpr bool IsHexChar(wchar_t c)
    {
        for (uint8_t i{ 0 }; i < 22; ++i)
        {
            if (c == hexChars[i]) { return true; }
        }
        return false;
    }
}

namespace winrt::simbolmudah_ui::implementation
{
    using namespace LibSimbolMudah;
    using namespace Windows::Foundation::Collections;

    UnicodePopup::UnicodePopup(SequenceDefinition const& seqdef) :
        seqdef(seqdef), hexCodes(single_threaded_observable_vector(std::vector<hstring>())) {}

    IObservableVector<hstring> UnicodePopup::HexCode() const { return this->hexCodes; }

    void UnicodePopup::ShowAnswer()
    {
        std::wstring numstring{};
        for (const auto& letter: this->hexCodes)
        {
            if (!IsHexChar(letter.c_str()[0]))
            {
                this->Result().Text(L"");
                this->Label().Text(L"Invalid hexadecimal number. Press enter to cancel.");
                return;
            }
            numstring.append(letter.c_str());
        }

        SequenceDescription desc{};
        try
        {
            const auto num{ std::stoul(numstring, nullptr, 16) };
            if (num > 0x10FFFF)
            {
                this->Result().Text(L"");
                this->Label().Text(L"Number out of range. Press enter to cancel.");
                return;
            }

            desc = this->seqdef.GetLocalizedName(num);
        }
        catch (const std::invalid_argument&)
        {
            this->Result().Text(L"");
            this->Label().Text(L"Invalid hexadecimal number. Press enter to cancel.");
            return;
        }
        catch (const std::out_of_range&)
        {
            this->Result().Text(L"");
            this->Label().Text(L"Number out of range. Press enter to cancel.");
            return;
        }
        catch (winrt::hresult_error const& e)
        {
            this->Result().Text(L"");
            this->Label().Text(e.message());
            return;
        }

        this->Result().Text(desc.result);
        this->Label().Text(desc.description);
    }

    void UnicodePopup::ResetState()
    {
        this->hexCodes.Clear();
        this->Result().Text(L"");
        this->Label().Text(L"Enter a hexadecimal number.");
    }
}
