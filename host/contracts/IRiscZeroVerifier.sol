// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.9;

import {reverseByteOrderUint32} from "./Util.sol";

struct Receipt {
    bytes seal;
    bytes32 claimDigest;
}

struct ReceiptClaim {
    bytes32 preStateDigest;
    bytes32 postStateDigest;
    ExitCode exitCode;
    bytes32 input;
    bytes32 output;
}

library ReceiptClaimLib {
    using OutputLib for Output;
    using SystemStateLib for SystemState;

    bytes32 constant TAG_DIGEST = sha256("risc0.ReceiptClaim");

    bytes32 constant SYSTEM_STATE_ZERO_DIGEST = 0xa3acc27117418996340b84e5a90f3ef4c49d22c79e44aad822ec9c313e1eb8e2;

    function ok(bytes32 imageId, bytes32 journalDigest) internal pure returns (ReceiptClaim memory) {
        return ReceiptClaim(
            imageId,
            SYSTEM_STATE_ZERO_DIGEST,
            ExitCode(SystemExitCode.Halted, 0),
            bytes32(0),
            Output(journalDigest, bytes32(0)).digest()
        );
    }

    function digest(ReceiptClaim memory claim) internal pure returns (bytes32) {
        return sha256(
            abi.encodePacked(
                TAG_DIGEST,
                claim.input,
                claim.preStateDigest,
                claim.postStateDigest,
                claim.output,
                uint32(claim.exitCode.system) << 24,
                uint32(claim.exitCode.user) << 24,
                uint16(4) << 8
            )
        );
    }
}

struct SystemState {
    uint32 pc;
    bytes32 merkle_root;
}

library SystemStateLib {
    bytes32 constant TAG_DIGEST = sha256("risc0.SystemState");

    function digest(SystemState memory state) internal pure returns (bytes32) {
        return sha256(
            abi.encodePacked(
                TAG_DIGEST,
                state.merkle_root,
                reverseByteOrderUint32(state.pc),
                uint16(1) << 8
            )
        );
    }
}

struct ExitCode {
    SystemExitCode system;
    uint8 user;
}

enum SystemExitCode {
    Halted,
    Paused,
    SystemSplit
}

struct Output {
    bytes32 journalDigest;
    bytes32 assumptionsDigest;
}

library OutputLib {
    bytes32 constant TAG_DIGEST = sha256("risc0.Output");

    function digest(Output memory output) internal pure returns (bytes32) {
        return sha256(
            abi.encodePacked(
                TAG_DIGEST,
                output.journalDigest,
                output.assumptionsDigest,
                uint16(2) << 8
            )
        );
    }
}

error VerificationFailed();

interface IRiscZeroVerifier {
    function verify(bytes calldata seal, bytes32 imageId, bytes32 journalDigest) external view;
    function verifyIntegrity(Receipt calldata receipt) external view;
}
