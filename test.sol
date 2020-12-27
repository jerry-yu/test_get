pragma solidity ^0.4.24;

contract Test {
    struct UC {
        string str1;
        uint256 ui256;
    }

    mapping(string => UC) keyUCMap;


    function newUC (
        string _key
        string _str1,
        uint256 _ui256
        ) public {

        UC memory uc = UC(
            {
                str1: _str1,
                ui256: _ui256
            }
        );
        keyUCMap[_key] = uc;
    }

    function getUC(string _key)
        public
        view
        returns (string,uint256)
    {
        UC memory uc = keyUCMap[_key];
        return (uc.str1, uc.ui256);
    }
}



