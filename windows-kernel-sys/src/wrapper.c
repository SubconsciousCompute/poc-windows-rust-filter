//
// Created by sn99 on 07-03-2023.
//

#include "wrapper.h"

PIO_STACK_LOCATION _IoGetCurrentIrpStackLocation(PIRP irp) {
    return IoGetCurrentIrpStackLocation(irp);
}
