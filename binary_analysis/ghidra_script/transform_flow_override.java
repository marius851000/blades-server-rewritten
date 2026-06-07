/*
 * Script to fix erroneous CALL_RETURN flow override on calls to
 * "AlsoMaybeNearInnerForEnd" (address: 0x011d55d4).
 *
 * Ghidra erroneously assumed this function was non-returning, which
 * caused it to set the CALL_RETURN (CALL_TERMINATOR) flow override on
 * all call sites. This script restores the default NONE override so
 * the decompiler treats the calls as normal function calls again.
 */

import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.address.AddressSet;
import ghidra.program.model.listing.FlowOverride;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.FunctionIterator;
import ghidra.program.model.listing.Instruction;
import ghidra.program.model.symbol.Reference;
import ghidra.program.model.symbol.ReferenceIterator;
import ghidra.program.disassemble.ReDisassembler;
import java.util.ArrayList;
import java.util.List;

public class transform_flow_override extends GhidraScript {

    private static final String TARGET_FUNCTION_NAME = "AlsoMaybeNearInnerForEnd";
    // Target function address: 0x011d55d4 (for reference/documentation purposes)

    @Override
    public void run() throws Exception {
        int fixedCount = 0;

        // Find ALL functions named "AlsoMaybeNearInnerForEnd" (may have thunks)
        FunctionIterator funcIter = currentProgram.getFunctionManager().getFunctions(true);
        List<Function> targetFuncs = new ArrayList<>();
        while (funcIter.hasNext()) {
            Function f = funcIter.next();
            if (f.getName().equals(TARGET_FUNCTION_NAME)) {
                targetFuncs.add(f);
            }
        }

        if (targetFuncs.isEmpty()) {
            print("ERROR: Could not find any function \"" + TARGET_FUNCTION_NAME + "\"\n");
            return;
        }

        print("Found " + targetFuncs.size() + " function(s) named \"" + TARGET_FUNCTION_NAME + "\":\n");
        for (Function f : targetFuncs) {
            print("  - " + f.getName() + " at " + f.getEntryPoint() + "\n");
        }

        print("\nSearching for calls with CALL_RETURN override...\n");

        // Process each function instance
        for (Function targetFunc : targetFuncs) {
            Address targetAddr = targetFunc.getEntryPoint();

            // Get all references TO this function instance (i.e., callers)
            ReferenceIterator refIter = currentProgram.getReferenceManager().getReferencesTo(targetAddr);

            while (refIter.hasNext()) {
                Reference ref = refIter.next();
                Address fromAddr = ref.getFromAddress();

                // Only care about call references
                if (!ref.getReferenceType().isCall()) {
                    continue;
                }

                Instruction instr = currentProgram.getListing().getInstructionAt(fromAddr);
                if (instr == null) {
                    continue;
                }

                FlowOverride currentOverride = instr.getFlowOverride();

                // Check if this call has CALL_RETURN (CALL_TERMINATOR) override
                if (currentOverride == FlowOverride.CALL_RETURN) {
                    print("Fixing: " + fromAddr + " - " + instr.getMnemonicString() +
                        " (was CALL_RETURN, setting to NONE)\n");
                    instr.setFlowOverride(FlowOverride.NONE);
                    fixedCount++;

                }

                //TODO: disassemble the result
            }
        }

        print("\nUnmarking all instances as non-returning...\n");
        for (Function f : targetFuncs) {
            print(f.hasNoReturn() + " - " + f.getName() + " at " + f.getEntryPoint() + "\n");
            if (f.hasNoReturn()) {
                f.setNoReturn(false);
                print("  - " + f.getName() + " at " + f.getEntryPoint() + ": noReturn = true -> false\n");
            }
        }

        print("\nDone. Fixed " + fixedCount + " call(s) to \"" + TARGET_FUNCTION_NAME + "\"\n");
    }
}
