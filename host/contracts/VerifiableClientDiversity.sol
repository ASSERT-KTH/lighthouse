// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

contract VerifiableClientDiversity is Ownable, ReentrancyGuard {
    enum ProofType {
        SGX_ATTESTATION,
        RISC0
    }

    ProofType public proofType;
    uint8 public allowedBlockDelay;
    address public sgxVerificationContract;
    address public risc0VerificationContract;

    bytes32[] public allowedVersions;
    mapping(bytes32 => uint256) public versionIndex;
    uint256 public reward;

    event RewardUpdated(uint256 oldReward, uint256 newReward);
    event AllowedDelayUpdated(uint8 oldDelay, uint8 newDelay);
    event VersionAdded(bytes32 versionHash);
    event VersionRemoved(bytes32 versionHash);
    event ProofSubmitted(address indexed submitter, bytes32 versionHash, uint256 reward);

    constructor(
        ProofType _proofType,
        uint8 _allowedBlockDelay
    ) Ownable(msg.sender) {
        proofType =  _proofType;
        allowedBlockDelay = _allowedBlockDelay;
        sgxVerificationContract = address(0x95175096a9B74165BE0ac84260cc14Fc1c0EF5FF); // Placeholder
        risc0VerificationContract = address(0x123463a4B065722E99115D6c222f267d9cABb524); // Placeholder
    }

    function setReward(uint256 _reward) external onlyOwner {
        uint256 oldReward = reward;
        reward = _reward;
        emit RewardUpdated(oldReward, _reward);
    }

    function setAllowedBlockDelay(uint8 _allowedDelay) external onlyOwner {
        uint8 oldDelay = allowedBlockDelay;
        allowedBlockDelay = _allowedDelay;
        emit AllowedDelayUpdated(oldDelay, _allowedDelay);
    }

    function addVersion(bytes32 versionHash) external onlyOwner {
        require(!isVersionAllowed(versionHash), "Version already allowed");
        allowedVersions.push(versionHash);
        emit VersionAdded(versionHash);
    }

    function removeVersion(bytes32 versionHash) external onlyOwner {
        require(isVersionAllowed(versionHash), "Version not found");
        for (uint256 i = 0; i < allowedVersions.length; i++) {
            if (allowedVersions[i] == versionHash) {
                allowedVersions[i] = allowedVersions[allowedVersions.length - 1];
                allowedVersions.pop();
                emit VersionRemoved(versionHash);
                break;
            }
        }
    }

    function getSupportedVersions() external view returns (bytes32[] memory) {
        return allowedVersions;
    }

    function submitProof(
        uint256 proofForBlock,
        bytes calldata seal,
        bytes32 imageId,
        bytes32 journalDigest
    ) external nonReentrant {
        require(isVersionAllowed(imageId), "Version not allowed");
        require(
            proofForBlock >= block.number && proofForBlock <= block.number + allowedBlockDelay,
            "Invalid block range"
        );

        bool isValid;
        if (proofType == ProofType.SGX_ATTESTATION) {
            isValid = verifySGXAttestation(seal);
        } else if (proofType == ProofType.RISC0) {
            isValid = verifyRISC0(seal, imageId, journalDigest);
        } else {
            revert("Unsupported proof type");
        }

        require(isValid, "Proof verification failed");

        versionIndex[imageId] += 1;

        (bool success, ) = msg.sender.call{value: reward}("");
        require(success, "Reward transfer failed");

        emit ProofSubmitted(msg.sender, imageId, reward);
    }

    function getMinority() external view returns (bytes32 versionHash) {
        require(allowedVersions.length > 0, "No versions available");

        uint256 minIndex = versionIndex[allowedVersions[0]];
        versionHash = allowedVersions[0];

        for (uint256 i = 1; i < allowedVersions.length; i++) {
            uint256 idx = versionIndex[allowedVersions[i]];
            if (idx < minIndex) {
                minIndex = idx;
                versionHash = allowedVersions[i];
            }
        }
    }

    function isVersionAllowed(bytes32 versionHash) public view returns (bool) {
        for (uint256 i = 0; i < allowedVersions.length; i++) {
            if (allowedVersions[i] == versionHash) {
                return true;
            }
        }
        return false;
    }

    function getVersionIndex(bytes32 versionHash) external view returns (uint256) {
        require(isVersionAllowed(versionHash), "Version not found");
        return versionIndex[versionHash];
    }

 function verifySGXAttestation(bytes calldata proofData) internal returns (bool) {
    // Construct the ABI encoding of the function signature
    bytes4 functionSelector = bytes4(keccak256("verifyAndAttestOnChain(bytes)"));

    // Build the complete call data
    bytes memory encodedCallData = abi.encodeWithSelector(functionSelector, proofData);

    // Use call to invoke the function
    (bool successCall, bytes memory result) = sgxVerificationContract.call{value: 0}(encodedCallData);

    // If the call fails, return false directly
    if (!successCall) {
        return false;
    }

    // Decode the return value (bool success, bytes memory output)
    (bool internalSuccess, ) = abi.decode(result, (bool, bytes));

    // Return the internal verification result
    return internalSuccess;
}

    function verifyRISC0(
        bytes calldata seal,
        bytes32 imageId,
        bytes32 journalDigest
    ) internal view returns (bool) {
        (bool success, bytes memory result) = risc0VerificationContract.staticcall(
            abi.encodeWithSignature("verify(bytes,bytes32,bytes32)", seal, imageId, journalDigest)
        );
        return success && abi.decode(result, (bool));
    }

    receive() external payable {}
}
