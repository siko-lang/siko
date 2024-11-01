%struct.siko_Unit = type {  }

define void @siko_Unit(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Unit, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 0, i1 false)
   ret void
}

define void @Main_main(ptr noundef %fn_result) {
block0:
   %i_0_5 = alloca %struct.siko_Unit, align 4
   %i_0_4 = alloca %struct.siko_Unit, align 4
   %i_0_3 = alloca %struct.siko_Unit, align 4
   %i_0_2 = alloca i64, align 8
   %i_0_1 = alloca i64, align 8
   call void @Bool_Bool_True(ptr %i_0_1)
   call void @Bool_Bool_False(ptr %i_0_2)
   call void @siko_Unit(ptr %i_0_3)
   call void @siko_Unit(ptr %i_0_4)
   call void @siko_Unit(ptr %i_0_5)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_0_5, i8 0, i1 false)
   ret void
}

define void @Bool_Bool_False(ptr noundef %fn_result) {
}

define void @Bool_Bool_True(ptr noundef %fn_result) {
block0:
   %var1 = alloca i64, align 8
   %var2 = load i64, ptr %var1, align 8
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Unit, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


