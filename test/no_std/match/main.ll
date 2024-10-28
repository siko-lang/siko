%struct.siko_Unit = type {  }

define void @siko_Unit(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Unit, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 0, i1 false)
   ret void
}

define void @Main_main(ptr noundef %fn_result) {
block0:
   %i_0_2 = alloca %struct.siko_Unit, align 4
   %i_0_1 = alloca %struct.siko_Unit, align 4
   call void @siko_Unit(ptr %i_0_1)
   call void @siko_Unit(ptr %i_0_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_0_2, i8 0, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Unit, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


