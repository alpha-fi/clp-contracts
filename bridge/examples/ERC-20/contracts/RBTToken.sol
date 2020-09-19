pragma solidity >=0.4.22 <0.7.0;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20Detailed.sol";

contract RBTToken is ERC20, ERC20Detailed {
    constructor(uint256 initialSupply) ERC20Detailed("Rainbow Bridge", "RBT", 18) public {
        _mint(msg.sender, initialSupply);
    }

    function mint(uint256 amount) public {
        _mint(msg.sender, amount);
    }

    
}